//! SSH socket I/O for the binary (§8): the connection pool, `ssh-agent`
//! delegation, and host-key verification wired to `russh`. The *decisions* -
//! command construction and host-key verdicts, come from `popush_core::ssh`;
//! this layer only performs I/O (D14).
//!
//! ## Verification note (Agent Rule 3)
//! This module is written against the pinned `russh` 0.45 / `russh-keys` 0.45 API,
//! read from the resolved crate source rather than trusted from memory: agent
//! authentication uses `authenticate_future` with the `AgentClient`'s `Signer`
//! impl (there is no `authenticate_publickey_with` in this line), and host-key
//! data is read via `PublicKey::name`/`fingerprint`/`public_key_base64`. It links
//! native crypto and only builds on the Linux target; `popush-core` holds
//! everything testable without a live SSH server. The integration tests (§23.3)
//! exercise this module against the containerised test VPS on the target.

use std::sync::Arc;
use std::time::{Duration, Instant};

use popush_core::command_log::CommandOutcome;
use popush_core::config::ServerConfig;
use popush_core::error::{AuthFailureReason, SshError};
use popush_core::ssh::{HostKeyDecision, HostKeyVerifier, KnownHost, RemoteCommand};

use russh::client;
use russh::ChannelMsg;
use russh_keys::agent::client::AgentClient;
use russh_keys::key::PublicKey;
use russh_keys::PublicKeyBase64;

/// A live, multiplexed SSH connection to one server (§8.1). Commands share this
/// single TCP connection via channels; a keepalive runs to detect a dead link.
pub struct SshPool {
    session: Arc<client::Handle<Handler>>,
    server: ServerConfig,
}

/// The `russh` client handler. Host-key checking delegates its *decision* to
/// `popush_core::ssh::hostkey` (§8.3); here we only extract the presented key's
/// algorithm, base64 blob, and fingerprint and consult `known_hosts`. Unknown and
/// mismatched keys are refused here so the UI can surface the fingerprint prompt
/// or the loud mismatch warning; the UI enforces the friction, never a one-click
/// accept.
struct Handler {
    host: String,
    known_hosts: Vec<KnownHost>,
}

#[async_trait::async_trait]
impl client::Handler for Handler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        let key_type = server_public_key.name().to_string();
        let key_base64 = server_public_key.public_key_base64();
        let fingerprint = format!("SHA256:{}", server_public_key.fingerprint());

        let verifier = HostKeyVerifier::new(&self.known_hosts);
        match verifier.verify(&self.host, &key_type, &key_base64, &fingerprint) {
            HostKeyDecision::Trusted => Ok(true),
            HostKeyDecision::Unknown { .. } | HostKeyDecision::Mismatch { .. } => Ok(false),
        }
    }
}

impl SshPool {
    /// Open a pooled connection to `server`, authenticating via `ssh-agent` (§8.2).
    /// Popush never handles a passphrase: if the agent offers no identity the
    /// server accepts, this returns a structured [`SshError`], never a generic one.
    pub async fn connect(
        server: ServerConfig,
        known_hosts: Vec<KnownHost>,
    ) -> Result<Self, SshError> {
        let config = Arc::new(client::Config {
            keepalive_interval: Some(Duration::from_secs(30)),
            ..Default::default()
        });
        let handler = Handler {
            host: server.host.clone(),
            known_hosts,
        };

        let addr = (server.host.as_str(), server.port);
        let mut session = client::connect(config, addr, handler).await.map_err(|e| {
            SshError::HostUnreachable {
                host: server.host.clone(),
                detail: e.to_string(),
            }
        })?;

        // Agent delegation: ask ssh-agent for identities and let it sign (§8.2).
        // A missing SSH_AUTH_SOCK is a specific, actionable error.
        let mut agent = AgentClient::connect_env()
            .await
            .map_err(|_| SshError::AuthFailed {
                reason: AuthFailureReason::NoAgentSocket,
            })?;

        let identities = agent
            .request_identities()
            .await
            .map_err(|_| SshError::AuthFailed {
                reason: AuthFailureReason::AgentRejected,
            })?;

        if identities.is_empty() {
            // The agent is running but holds nothing usable for this key (§8.2).
            return Err(SshError::KeyNotInAgent {
                path: server.identity_file.clone(),
            });
        }

        // `authenticate_future` consumes and returns the agent (the `Signer`), so
        // we thread it through each attempt until one succeeds.
        let mut authenticated = false;
        for key in identities {
            let (returned_agent, result) = session
                .authenticate_future(&server.username, key, agent)
                .await;
            agent = returned_agent;
            if matches!(result, Ok(true)) {
                authenticated = true;
                break;
            }
        }

        if !authenticated {
            return Err(SshError::AuthFailed {
                reason: AuthFailureReason::AllMethodsExhausted,
            });
        }

        Ok(Self {
            session: Arc::new(session),
            server,
        })
    }

    /// Run a [`RemoteCommand`] on this server (§8.4). Returns a [`CommandOutcome`]
    /// carrying the exact command shown in the command log (D8).
    pub async fn exec(&self, command: RemoteCommand) -> Result<CommandOutcome, SshError> {
        let rendered = command.render();
        let display = command.display();
        let start = Instant::now();

        let mut channel = self
            .session
            .channel_open_session()
            .await
            .map_err(|_| SshError::ConnectionLost)?;
        channel
            .exec(true, rendered.as_bytes())
            .await
            .map_err(|_| SshError::ConnectionLost)?;

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut exit_code = 0i32;

        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { ref data } => stdout.extend_from_slice(&data[..]),
                // ext == 1 is stderr (SSH_EXTENDED_DATA_STDERR).
                ChannelMsg::ExtendedData { ref data, ext } if ext == 1 => {
                    stderr.extend_from_slice(&data[..])
                }
                ChannelMsg::ExitStatus { exit_status } => exit_code = exit_status as i32,
                ChannelMsg::Eof | ChannelMsg::Close => break,
                _ => {}
            }
        }

        Ok(CommandOutcome {
            exit_code,
            stdout: String::from_utf8_lossy(&stdout).into_owned(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
            duration_ms: start.elapsed().as_millis() as u64,
            command_display: display,
        })
    }

    /// The server this pool connects to.
    pub fn server(&self) -> &ServerConfig {
        &self.server
    }
}
