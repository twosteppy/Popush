use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use popush_core::command_log::CommandOutcome;
use popush_core::config::ServerConfig;
use popush_core::error::{AuthFailureReason, SshError};
use popush_core::ssh::{
    known_hosts_lookup_key, HostKeyDecision, HostKeyVerifier, KnownHost, RemoteCommand,
};

const MAX_STREAM_BYTES: usize = 10 * 1024 * 1024;

/// Append `data` to `buf`, stopping at [`MAX_STREAM_BYTES`] and setting
/// `truncated` if the cap is hit, so a runaway command cannot exhaust memory.
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
    /// When true, an unknown host is accepted on first use and recorded in
    /// `accepted` so the caller can persist it to known_hosts. A *changed* key is
    /// still refused, since that is the man-in-the-middle case.
    tofu: bool,
    decision: Arc<Mutex<Option<HostKeyDecision>>>,
    accepted: Arc<Mutex<Option<KnownHost>>>,
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
        let trusted = match &decision {
            HostKeyDecision::Trusted => true,
            HostKeyDecision::Unknown { .. } if self.tofu => {
                *self.accepted.lock().unwrap() = Some(KnownHost {
                    host: lookup,
                    key_type: key_type.clone(),
                    key_base64: key_base64.clone(),
                });
                true
            }
            _ => false,
        };
        *self.decision.lock().unwrap() = Some(decision);
        Ok(trusted)
    }
}

impl SshPool {
    /// Connect, refusing any host that is not already trusted in known_hosts.
    pub async fn connect(
        server: ServerConfig,
        known_hosts: Vec<KnownHost>,
        password: Option<&str>,
    ) -> Result<Self, SshError> {
        Ok(Self::connect_inner(server, known_hosts, false, password)
            .await?
            .0)
    }

    /// Connect with trust-on-first-use: an unknown host is accepted and returned
    /// as a [`KnownHost`] so the caller can persist it; a changed key is refused.
    pub async fn connect_tofu(
        server: ServerConfig,
        known_hosts: Vec<KnownHost>,
        password: Option<&str>,
    ) -> Result<(Self, Option<KnownHost>), SshError> {
        Self::connect_inner(server, known_hosts, true, password).await
    }

    async fn connect_inner(
        server: ServerConfig,
        known_hosts: Vec<KnownHost>,
        tofu: bool,
        password: Option<&str>,
    ) -> Result<(Self, Option<KnownHost>), SshError> {
        let config = Arc::new(client::Config {
            keepalive_interval: Some(Duration::from_secs(30)),
            ..Default::default()
        });
        let decision = Arc::new(Mutex::new(None));
        let accepted = Arc::new(Mutex::new(None));
        let handler = Handler {
            host: server.host.clone(),
            port: server.port,
            known_hosts,
            tofu,
            decision: decision.clone(),
            accepted: accepted.clone(),
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

        // Auth order: every key in the ssh-agent, then the configured identity
        // file, then the usual default keys. The key-file fallback means a
        // passphrase-free key works without ever running ssh-add.
        let mut authenticated = false;
        let mut agent_had_keys = false;

        if let Ok(mut agent) = AgentClient::connect_env().await {
            if let Ok(identities) = agent.request_identities().await {
                agent_had_keys = !identities.is_empty();
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
            }
        }

        if !authenticated {
            for path in candidate_key_files(&server) {
                let Ok(key) = russh_keys::load_secret_key(&path, None) else {
                    // Unreadable or passphrase-protected; the agent is the
                    // only way in for those, so move on.
                    continue;
                };
                let ok = session
                    .authenticate_publickey(&server.username, Arc::new(key))
                    .await
                    .unwrap_or(false);
                if ok {
                    authenticated = true;
                    break;
                }
            }
        }

        // A password the user typed into the app for this session. Held in
        // memory only, never written anywhere.
        if !authenticated {
            if let Some(pw) = password {
                authenticated = session
                    .authenticate_password(&server.username, pw)
                    .await
                    .unwrap_or(false);
                if !authenticated {
                    return Err(SshError::AuthFailed {
                        reason: AuthFailureReason::AllMethodsExhausted,
                    });
                }
            }
        }

        if !authenticated {
            return Err(if agent_had_keys {
                SshError::AuthFailed {
                    reason: AuthFailureReason::AllMethodsExhausted,
                }
            } else {
                SshError::KeyNotInAgent {
                    path: server.identity_file.clone(),
                }
            });
        }

        let new_host = accepted.lock().unwrap().take();
        Ok((
            Self {
                session: Arc::new(session),
                server,
            },
            new_host,
        ))
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

    /// Like [`exec`], but calls `on_line` with each output line as it arrives
    /// (so a long build streams live instead of appearing frozen) and polls
    /// `is_cancelled` while waiting. Returns `Ok(None)` when cancelled
    /// mid-command: the channel is dropped, which tells the remote to stop.
    pub async fn exec_streaming(
        &self,
        command: RemoteCommand,
        mut on_line: impl FnMut(&str, bool),
        is_cancelled: impl Fn() -> bool,
    ) -> Result<Option<CommandOutcome>, SshError> {
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
        // Bytes seen but not yet ending in a newline, held back until they do.
        let mut stdout_pending = String::new();
        let mut stderr_pending = String::new();

        loop {
            // Wake at least every 200ms so cancellation is honoured promptly
            // even while the remote command is producing no output. `wait` is
            // cancel-safe (a dropped poll loses no message), so the timeout is
            // safe to race against it.
            match tokio::time::timeout(Duration::from_millis(200), channel.wait()).await {
                Ok(Some(ChannelMsg::Data { ref data })) => {
                    append_capped(&mut stdout, &mut stdout_truncated, &data[..]);
                    stdout_pending.push_str(&String::from_utf8_lossy(&data[..]));
                    while let Some(idx) = stdout_pending.find('\n') {
                        let line = stdout_pending[..idx].trim_end_matches('\r').to_string();
                        on_line(&line, false);
                        stdout_pending.drain(..=idx);
                    }
                }
                Ok(Some(ChannelMsg::ExtendedData { ref data, ext: 1 })) => {
                    append_capped(&mut stderr, &mut stderr_truncated, &data[..]);
                    stderr_pending.push_str(&String::from_utf8_lossy(&data[..]));
                    while let Some(idx) = stderr_pending.find('\n') {
                        let line = stderr_pending[..idx].trim_end_matches('\r').to_string();
                        on_line(&line, true);
                        stderr_pending.drain(..=idx);
                    }
                }
                Ok(Some(ChannelMsg::ExitStatus { exit_status })) => exit_code = exit_status as i32,
                Ok(Some(_)) => {}
                Ok(None) => break,
                Err(_elapsed) => {
                    if is_cancelled() {
                        return Ok(None);
                    }
                }
            }
        }

        // Emit whatever trailing text never got its own newline.
        if !stdout_pending.is_empty() {
            on_line(stdout_pending.trim_end_matches('\r'), false);
        }
        if !stderr_pending.is_empty() {
            on_line(stderr_pending.trim_end_matches('\r'), true);
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

        Ok(Some(CommandOutcome {
            exit_code,
            stdout,
            stderr,
            duration_ms: start.elapsed().as_millis() as u64,
            command_display: display,
        }))
    }

    pub fn server(&self) -> &ServerConfig {
        &self.server
    }
}

/// Private keys worth trying, existing files only: the configured identity
/// first, then the standard names in ~/.ssh.
fn candidate_key_files(server: &ServerConfig) -> Vec<std::path::PathBuf> {
    let home = directories::UserDirs::new().map(|d| d.home_dir().to_path_buf());
    let mut paths = Vec::new();
    let mut push = |p: std::path::PathBuf| {
        if p.is_file() && !paths.contains(&p) {
            paths.push(p);
        }
    };

    push(expand_tilde(&server.identity_file, home.as_deref()));
    if let Some(h) = &home {
        for name in ["id_ed25519", "id_rsa", "id_ecdsa"] {
            push(h.join(".ssh").join(name));
        }
    }
    paths
}

fn expand_tilde(path: &std::path::Path, home: Option<&std::path::Path>) -> std::path::PathBuf {
    let Some(home) = home else {
        return path.to_path_buf();
    };
    match path.strip_prefix("~") {
        Ok(rest) => home.join(rest),
        Err(_) => path.to_path_buf(),
    }
}
