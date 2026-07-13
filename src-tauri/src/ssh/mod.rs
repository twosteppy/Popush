//! SSH socket I/O for the binary: the connection pool, `ssh-agent`
//! delegation, and host-key verification wired to `russh`. The *decisions* -
//! command construction and host-key verdicts, come from `popush_core::ssh`;
//! this layer only performs I/O.
//!
//! ## Verification note
//! This module is written against the pinned `russh` 0.45 / `russh-keys` 0.45 API,
//! read from the resolved crate source rather than trusted from memory: agent
//! authentication uses `authenticate_future` with the `AgentClient`'s `Signer`
//! impl (there is no `authenticate_publickey_with` in this line), and host-key
//! data is read via `PublicKey::name`/`fingerprint`/`public_key_base64`. It links
//! native crypto and only builds on the Linux target; `popush-core` holds
//! everything testable without a live SSH server. The integration tests
//! exercise this module against the containerised test VPS on the target.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use popush_core::command_log::CommandOutcome;
use popush_core::config::ServerConfig;
use popush_core::error::{AuthFailureReason, SshError};
use popush_core::ssh::{
    known_hosts_lookup_key, HostKeyDecision, HostKeyVerifier, KnownHost, RemoteCommand,
};

/// Hard cap on bytes captured per stream from a single command. A compromised or
/// hostile server could otherwise stream unbounded output (e.g. `yes`) and drive
/// the app to OOM. Beyond this, output is dropped and a marker is appended.
const MAX_STREAM_BYTES: usize = 10 * 1024 * 1024;

use russh::client;
use russh::ChannelMsg;
use russh_keys::agent::client::AgentClient;
use russh_keys::key::PublicKey;
use russh_keys::PublicKeyBase64;

/// A live, multiplexed SSH connection to one server. Commands share this
/// single TCP connection via channels; a keepalive runs to detect a dead link.
pub struct SshPool {
    session: Arc<client::Handle<Handler>>,
    server: ServerConfig,
}

/// The `russh` client handler. Host-key checking delegates its *decision* to
/// `popush_core::ssh::hostkey`; here we only extract the presented key's
/// algorithm, base64 blob, and fingerprint and consult `known_hosts`. Unknown and
/// mismatched keys are refused here so the UI can surface the fingerprint prompt
/// or the loud mismatch warning; the UI enforces the friction, never a one-click
/// accept.
struct Handler {
    host: String,
    port: u16,
    known_hosts: Vec<KnownHost>,
    /// The verdict from the most recent `check_server_key`, shared with
    /// `connect` so a refusal can be reported as the specific host-key error
    /// (unknown host, or the loud key-changed/MITM warning) rather than a
    /// generic "host unreachable".
    decision: Arc<Mutex<Option<HostKeyDecision>>>,
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

        // Match known_hosts exactly as OpenSSH does, keying on the port so a pin
        // for port 22 can never be reused to trust another port of the same host.
        let lookup = known_hosts_lookup_key(&self.host, self.port);
        let verifier = HostKeyVerifier::new(&self.known_hosts);
        let decision = verifier.verify(&lookup, &key_type, &key_base64, &fingerprint);
        let trusted = matches!(decision, HostKeyDecision::Trusted);
        *self.decision.lock().unwrap() = Some(decision);
        Ok(trusted)
    }
}

impl SshPool {
    /// Open a pooled connection to `server`, authenticating via `ssh-agent`.
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
        let decision = Arc::new(Mutex::new(None));
        let handler = Handler {
            host: server.host.clone(),
            port: server.port,
            known_hosts,
            decision: decision.clone(),
        };

        let addr = (server.host.as_str(), server.port);
        let mut session = match client::connect(config, addr, handler).await {
            Ok(session) => session,
            Err(e) => {
                // If the handshake was refused because host-key verification
                // failed, surface the specific verdict so the UI can show the
                // fingerprint prompt or the loud key-changed/MITM warning.
                let verdict = decision.lock().unwrap().take();
                return Err(match verdict {
                    Some(HostKeyDecision::Mismatch { expected, got }) => {
                        SshError::HostKeyMismatch {
                            host: server.host.clone(),
                            expected,
                            got,
                        }
                    }
                    Some(HostKeyDecision::Unknown { fingerprint }) => SshError::HostKeyUnknown {
                        host: server.host.clone(),
                        fingerprint,
                    },
                    _ => SshError::HostUnreachable {
                        host: server.host.clone(),
                        detail: e.to_string(),
                    },
                });
            }
        };

        // Agent delegation: ask ssh-agent for identities and let it sign.
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
            // The agent is running but holds nothing usable for this key.
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

    /// Run a [`RemoteCommand`] on this server. Returns a [`CommandOutcome`]
    /// carrying the exact command shown in the command log.
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
        let mut stdout_truncated = false;
        let mut stderr_truncated = false;
        let mut exit_code = 0i32;

        // Append `data` to `buf` but never past the cap; flag truncation instead
        // of growing without bound.
        fn append_capped(buf: &mut Vec<u8>, truncated: &mut bool, data: &[u8]) {
            if buf.len() >= MAX_STREAM_BYTES {
                *truncated = true;
                return;
            }
            let room = MAX_STREAM_BYTES - buf.len();
            if data.len() > room {
                buf.extend_from_slice(&data[..room]);
                *truncated = true;
            } else {
                buf.extend_from_slice(data);
            }
        }

        // Drain the channel to completion. We do NOT break on `Eof`/`Close`: SSH
        // servers may send `ExitStatus` *after* `Eof` (and sometimes after
        // `Close`), so breaking early loses the real exit code and reports 0. The
        // loop ends naturally when `wait` returns `None`, by which point every
        // message, including a trailing exit status, has been seen. Verified
        // against a live sshd: `exit 7` now reports 7, not 0.
        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { ref data } => {
                    append_capped(&mut stdout, &mut stdout_truncated, &data[..])
                }
                // ext == 1 is stderr (SSH_EXTENDED_DATA_STDERR).
                ChannelMsg::ExtendedData { ref data, ext: 1 } => {
                    append_capped(&mut stderr, &mut stderr_truncated, &data[..])
                }
                ChannelMsg::ExitStatus { exit_status } => exit_code = exit_status as i32,
                _ => {}
            }
        }

        let marker = "\n[output truncated by Popush: exceeded 10 MiB]";
        let mut stdout = String::from_utf8_lossy(&stdout).into_owned();
        if stdout_truncated {
            stdout.push_str(marker);
        }
        let mut stderr = String::from_utf8_lossy(&stderr).into_owned();
        if stderr_truncated {
            stderr.push_str(marker);
        }

        Ok(CommandOutcome {
            exit_code,
            stdout,
            stderr,
            duration_ms: start.elapsed().as_millis() as u64,
            command_display: display,
        })
    }

    /// The server this pool connects to.
    pub fn server(&self) -> &ServerConfig {
        &self.server
    }
}
