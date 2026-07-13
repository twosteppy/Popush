use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use popush_core::command_log::CommandOutcome;
use popush_core::config::ServerConfig;
use popush_core::error::{AuthFailureReason, SshError};
use popush_core::ssh::{
    known_hosts_lookup_key, HostKeyDecision, HostKeyVerifier, KnownHost, RemoteCommand,
};

const MAX_STREAM_BYTES: usize = 10 * 1024 * 1024;

use russh::client;
use russh::ChannelMsg;
use russh_keys::agent::client::AgentClient;
use russh_keys::key::PublicKey;
use russh_keys::PublicKeyBase64;

pub struct SshPool {
    session: Arc<client::Handle<Handler>>,
    server: ServerConfig,
}

struct Handler {
    host: String,
    port: u16,
    known_hosts: Vec<KnownHost>,
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

        let lookup = known_hosts_lookup_key(&self.host, self.port);
        let verifier = HostKeyVerifier::new(&self.known_hosts);
        let decision = verifier.verify(&lookup, &key_type, &key_base64, &fingerprint);
        let trusted = matches!(decision, HostKeyDecision::Trusted);
        *self.decision.lock().unwrap() = Some(decision);
        Ok(trusted)
    }
}

impl SshPool {
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
            return Err(SshError::KeyNotInAgent {
                path: server.identity_file.clone(),
            });
        }

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

        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { ref data } => {
                    append_capped(&mut stdout, &mut stdout_truncated, &data[..])
                }
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

    pub fn server(&self) -> &ServerConfig {
        &self.server
    }
}
