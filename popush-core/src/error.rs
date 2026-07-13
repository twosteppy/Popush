use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct UserMessage {
    pub headline: String,
    pub consequence: String,
    pub next_action: NextAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NextAction {
    RunCommand { command: String },
    OpenFlow { flow: String, label: String },
    Retry,
    Advice { text: String },
}

#[derive(Debug, thiserror::Error, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum AppError {
    #[error("ssh: {0}")]
    Ssh(SshError),
    #[error("git: {0}")]
    Git(GitError),
    #[error("adapter: {0}")]
    Adapter(AdapterError),
    #[error("config: {0}")]
    Config(ConfigError),
    #[error("pipeline: {0}")]
    Pipeline(PipelineError),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize, TS)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum AuthFailureReason {
    #[error("no identity in ssh-agent was accepted by the server")]
    AgentRejected,
    #[error("SSH_AUTH_SOCK is not set; no ssh-agent is running")]
    NoAgentSocket,
    #[error("the server accepted none of the offered authentication methods")]
    AllMethodsExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize, TS)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum SshError {
    #[error("host {host} is unreachable: {detail}")]
    HostUnreachable { host: String, detail: String },
    #[error("authentication failed: {reason}")]
    AuthFailed { reason: AuthFailureReason },
    #[error("host key mismatch for {host}")]
    HostKeyMismatch {
        host: String,
        expected: String,
        got: String,
    },
    #[error("host {host} is unknown; fingerprint {fingerprint}")]
    HostKeyUnknown { host: String, fingerprint: String },
    #[error("key {path} has a passphrase and is not in the agent")]
    KeyNotInAgent { path: PathBuf },
    #[error("key file {path} not found")]
    KeyNotFound { path: PathBuf },
    #[error("command exited {exit_code}")]
    CommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },
    #[error("connection lost")]
    ConnectionLost,
    #[error("timed out after {after_ms}ms")]
    Timeout { after_ms: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize, TS)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum GitError {
    #[error("{count} files have unresolved merge conflicts")]
    MergeConflicts { count: usize, files: Vec<PathBuf> },
    #[error("detached HEAD")]
    DetachedHead,
    #[error("branch {branch} has no upstream")]
    NoUpstream { branch: String },
    #[error("remote {url} is HTTPS, not SSH")]
    HttpsRemote { url: String },
    #[error("remote {url} is not an SSH remote")]
    NonSshRemote { url: String },
    #[error("push rejected: non-fast-forward")]
    PushRejectedNonFastForward,
    #[error("push rejected: permission denied")]
    PushRejectedPermission,
    #[error("git operation failed: {detail}")]
    Operation { detail: String },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize, TS)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum AdapterError {
    #[error("could not parse {tool} output: {detail}")]
    Unparseable { tool: String, detail: String },
    #[error("{operation} is not supported for {service_type} sites")]
    Unsupported {
        operation: String,
        service_type: String,
    },
    #[error("remote command failed: {0}")]
    Ssh(SshError),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize, TS)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ConfigError {
    #[error("cannot read config at {path}: {detail}")]
    Unreadable { path: PathBuf, detail: String },
    #[error("config is not valid TOML: {detail}")]
    Malformed { detail: String },
    #[error("field `{field}` is invalid: {problem}")]
    InvalidField { field: String, problem: String },
    #[error("config schema version {found} is newer than supported {supported}")]
    SchemaTooNew { found: u32, supported: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize, TS)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum PipelineError {
    #[error("step {step} failed")]
    StepFailed { step: String, detail: String },
    #[error("cancelled at step {step}")]
    Cancelled { step: String, mid_mutation: bool },
}

impl AppError {
    pub fn user_message(&self) -> UserMessage {
        match self {
            AppError::Ssh(e) => e.user_message(),
            AppError::Git(e) => e.user_message(),
            AppError::Adapter(e) => e.user_message(),
            AppError::Config(e) => e.user_message(),
            AppError::Pipeline(e) => e.user_message(),
        }
    }
}

impl SshError {
    pub fn user_message(&self) -> UserMessage {
        match self {
            SshError::HostUnreachable { host, detail } => UserMessage {
                headline: format!("Cannot reach {host}."),
                consequence: "Popush cannot check or change anything on this server until it responds.".into(),
                next_action: NextAction::Advice {
                    text: format!("Check the server is up and reachable ({detail}), then retry."),
                },
            },
            SshError::AuthFailed { reason } => UserMessage {
                headline: "The server refused authentication.".into(),
                consequence: "Popush cannot run commands on this server.".into(),
                next_action: match reason {
                    AuthFailureReason::NoAgentSocket => NextAction::RunCommand {
                        command: "eval \"$(ssh-agent -s)\" && ssh-add".into(),
                    },
                    _ => NextAction::OpenFlow {
                        flow: "wizard".into(),
                        label: "Run the setup wizard".into(),
                    },
                },
            },
            SshError::HostKeyMismatch { host, expected, got } => UserMessage {
                headline: format!("The host key for {host} has changed."),
                consequence: "This can mean someone is intercepting the connection. Popush will not connect.".into(),
                next_action: NextAction::Advice {
                    text: format!(
                        "Expected {expected} but got {got}. If you changed the server yourself, remove the old key from ~/.ssh/known_hosts by hand. Otherwise, do not connect."
                    ),
                },
            },
            SshError::HostKeyUnknown { host, fingerprint } => UserMessage {
                headline: format!("{host} is a new host."),
                consequence: "Popush has never connected here before and cannot verify its identity for you.".into(),
                next_action: NextAction::Advice {
                    text: format!("Confirm this fingerprint matches your server: {fingerprint}"),
                },
            },
            SshError::KeyNotInAgent { path } => UserMessage {
                headline: format!("The key {} has a passphrase and is not loaded in your SSH agent.", path.display()),
                consequence: "Popush does not handle passphrases, by design.".into(),
                next_action: NextAction::RunCommand {
                    command: format!("ssh-add {}", path.display()),
                },
            },
            SshError::KeyNotFound { path } => UserMessage {
                headline: format!("No SSH key at {}.", path.display()),
                consequence: "Popush cannot authenticate to this server.".into(),
                next_action: NextAction::OpenFlow {
                    flow: "wizard".into(),
                    label: "Create a key with the wizard".into(),
                },
            },
            SshError::CommandFailed { command, exit_code, stderr } => UserMessage {
                headline: format!("A remote command exited with code {exit_code}."),
                consequence: "The command did not complete. Nothing was assumed to have changed.".into(),
                next_action: NextAction::Advice {
                    text: format!("Command: {command}\n{stderr}"),
                },
            },
            SshError::ConnectionLost => UserMessage {
                headline: "The connection to the server dropped.".into(),
                consequence: "Any in-flight command may or may not have finished; Popush cannot know.".into(),
                next_action: NextAction::Retry,
            },
            SshError::Timeout { after_ms } => UserMessage {
                headline: format!("The server did not respond within {}s.", after_ms / 1000),
                consequence: "Popush stopped waiting rather than hang the app.".into(),
                next_action: NextAction::Retry,
            },
        }
    }
}

impl GitError {
    pub fn user_message(&self) -> UserMessage {
        match self {
            GitError::MergeConflicts { count, files } => UserMessage {
                headline: format!(
                    "This repository has unresolved merge conflicts in {count} files."
                ),
                consequence: "Popush does not resolve conflicts.".into(),
                next_action: NextAction::Advice {
                    text: format!(
                        "Fix them in your editor, then come back.\n{}",
                        files.iter().map(|f| f.display().to_string()).collect::<Vec<_>>().join("\n")
                    ),
                },
            },
            GitError::DetachedHead => UserMessage {
                headline: "You are not on a branch (detached HEAD).".into(),
                consequence: "Commits made now would not belong to any branch.".into(),
                next_action: NextAction::Advice {
                    text: "Check out a branch before committing.".into(),
                },
            },
            GitError::NoUpstream { branch } => UserMessage {
                headline: format!("Branch `{branch}` has no upstream."),
                consequence: "Popush does not know where to push it.".into(),
                next_action: NextAction::RunCommand {
                    command: format!("git push --set-upstream origin {branch}"),
                },
            },
            GitError::HttpsRemote { url } => UserMessage {
                headline: "This repository uses an HTTPS remote.".into(),
                consequence: "GitHub removed password auth for HTTPS in 2021, so an HTTPS push needs a token. Popush does not collect tokens.".into(),
                next_action: NextAction::OpenFlow {
                    flow: "wizard".into(),
                    label: format!("Convert {url} to SSH in the wizard"),
                },
            },
            GitError::NonSshRemote { url } => UserMessage {
                headline: format!("The remote `{url}` is not an SSH remote."),
                consequence: "Popush only pushes over SSH, so it will not push to this remote.".into(),
                next_action: NextAction::Advice {
                    text: "Point origin at an SSH remote (git@host:owner/repo.git), then try again.".into(),
                },
            },
            GitError::PushRejectedNonFastForward => UserMessage {
                headline: "Push rejected. The remote has commits you do not have.".into(),
                consequence: "Your local branch is behind the remote.".into(),
                next_action: NextAction::Advice {
                    text: "Pull and merge them first.".into(),
                },
            },
            GitError::PushRejectedPermission => UserMessage {
                headline: "Push rejected: permission denied.".into(),
                consequence: "Your SSH key may not be registered with GitHub.".into(),
                next_action: NextAction::OpenFlow {
                    flow: "wizard".into(),
                    label: "Run the setup wizard".into(),
                },
            },
            GitError::Operation { detail } => UserMessage {
                headline: "A git operation did not complete.".into(),
                consequence: "Your working tree was not changed by Popush.".into(),
                next_action: NextAction::Advice { text: detail.clone() },
            },
        }
    }
}

impl AdapterError {
    pub fn user_message(&self) -> UserMessage {
        match self {
            AdapterError::Unparseable { tool, detail } => UserMessage {
                headline: format!("Popush could not read the status from {tool}."),
                consequence:
                    "The site's real state is unknown, so Popush shows amber rather than guess."
                        .into(),
                next_action: NextAction::Advice {
                    text: detail.clone(),
                },
            },
            AdapterError::Unsupported {
                operation,
                service_type,
            } => UserMessage {
                headline: format!("{operation} is not available for {service_type} sites."),
                consequence: "This service type has no such action, so Popush does not offer it."
                    .into(),
                next_action: NextAction::Advice {
                    text: "This is expected, not a fault.".into(),
                },
            },
            AdapterError::Ssh(e) => e.user_message(),
        }
    }
}

impl ConfigError {
    pub fn user_message(&self) -> UserMessage {
        match self {
            ConfigError::Unreadable { path, detail } => UserMessage {
                headline: format!("Popush could not read {}.", path.display()),
                consequence: "Without a config, Popush has no servers or sites to show.".into(),
                next_action: NextAction::Advice { text: detail.clone() },
            },
            ConfigError::Malformed { detail } => UserMessage {
                headline: "Your config file is not valid TOML.".into(),
                consequence: "Popush cannot load any servers until it parses.".into(),
                next_action: NextAction::Advice {
                    text: format!("Fix the syntax error and reload: {detail}"),
                },
            },
            ConfigError::InvalidField { field, problem } => UserMessage {
                headline: format!("The `{field}` setting is not valid."),
                consequence: "Popush will not load a config it does not understand, to avoid acting on a guess.".into(),
                next_action: NextAction::Advice {
                    text: format!("{field}: {problem}"),
                },
            },
            ConfigError::SchemaTooNew { found, supported } => UserMessage {
                headline: "This config was written by a newer Popush.".into(),
                consequence: format!("It uses schema {found}; this build understands up to {supported}."),
                next_action: NextAction::Advice {
                    text: "Update Popush, or edit the config's schema_version by hand.".into(),
                },
            },
        }
    }
}

impl PipelineError {
    pub fn user_message(&self) -> UserMessage {
        match self {
            PipelineError::StepFailed { step, detail } => UserMessage {
                headline: format!("The {step} step did not complete."),
                consequence: "Ship It stopped at this step; later steps did not run.".into(),
                next_action: NextAction::Advice {
                    text: detail.clone(),
                },
            },
            PipelineError::Cancelled { step, mid_mutation } => UserMessage {
                headline: format!("You cancelled Ship It during the {step} step."),
                consequence: if *mid_mutation {
                    "This step changes the server, so your site may be in an in-between state. Completed steps were not undone.".into()
                } else {
                    "No changes had been made at this point.".into()
                },
                next_action: NextAction::Advice {
                    text: "Check the site's status before trying again.".into(),
                },
            },
        }
    }
}

impl From<SshError> for AppError {
    fn from(e: SshError) -> Self {
        AppError::Ssh(e)
    }
}
impl From<GitError> for AppError {
    fn from(e: GitError) -> Self {
        AppError::Git(e)
    }
}
impl From<AdapterError> for AppError {
    fn from(e: AdapterError) -> Self {
        AppError::Adapter(e)
    }
}
impl From<ConfigError> for AppError {
    fn from(e: ConfigError) -> Self {
        AppError::Config(e)
    }
}
impl From<PipelineError> for AppError {
    fn from(e: PipelineError) -> Self {
        AppError::Pipeline(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BANNED: &[&str] = &["Deploy failed", "Something went wrong"];

    fn assert_answers_three_questions(m: &UserMessage) {
        assert!(
            !m.headline.trim().is_empty(),
            "headline (what happened) empty"
        );
        assert!(
            !m.consequence.trim().is_empty(),
            "consequence (what it means) empty"
        );
        for banned in BANNED {
            assert!(!m.headline.contains(banned), "banned string in headline");
            assert!(
                !m.consequence.contains(banned),
                "banned string in consequence"
            );
        }
    }

    #[test]
    fn every_ssh_variant_answers_the_three_questions() {
        let variants = [
            SshError::HostUnreachable {
                host: "h".into(),
                detail: "refused".into(),
            },
            SshError::AuthFailed {
                reason: AuthFailureReason::AgentRejected,
            },
            SshError::AuthFailed {
                reason: AuthFailureReason::NoAgentSocket,
            },
            SshError::HostKeyMismatch {
                host: "h".into(),
                expected: "a".into(),
                got: "b".into(),
            },
            SshError::HostKeyUnknown {
                host: "h".into(),
                fingerprint: "SHA256:x".into(),
            },
            SshError::KeyNotInAgent { path: "/k".into() },
            SshError::KeyNotFound { path: "/k".into() },
            SshError::CommandFailed {
                command: "c".into(),
                exit_code: 1,
                stderr: "e".into(),
            },
            SshError::ConnectionLost,
            SshError::Timeout { after_ms: 10000 },
        ];
        for v in variants {
            assert_answers_three_questions(&v.user_message());
        }
    }

    #[test]
    fn every_git_variant_answers_the_three_questions() {
        let variants = [
            GitError::MergeConflicts {
                count: 3,
                files: vec!["a".into()],
            },
            GitError::DetachedHead,
            GitError::NoUpstream {
                branch: "feature/x".into(),
            },
            GitError::HttpsRemote {
                url: "https://github.com/u/r.git".into(),
            },
            GitError::PushRejectedNonFastForward,
            GitError::PushRejectedPermission,
            GitError::Operation { detail: "d".into() },
        ];
        for v in variants {
            assert_answers_three_questions(&v.user_message());
        }
    }

    #[test]
    fn passphrase_message_matches_spec_8_2() {
        let m = SshError::KeyNotInAgent {
            path: "/home/u/.ssh/id_ed25519".into(),
        }
        .user_message();
        assert!(m
            .headline
            .contains("has a passphrase and is not loaded in your SSH agent"));
        assert!(m
            .consequence
            .contains("does not handle passphrases, by design"));
        assert_eq!(
            m.next_action,
            NextAction::RunCommand {
                command: "ssh-add /home/u/.ssh/id_ed25519".into()
            }
        );
    }

    #[test]
    fn config_errors_name_the_field() {
        let m = ConfigError::InvalidField {
            field: "port".into(),
            problem: "must be 1-65535".into(),
        }
        .user_message();
        assert!(m.headline.contains("port"));
        assert!(
            m.next_action
                == NextAction::Advice {
                    text: "port: must be 1-65535".into()
                }
        );
    }

    #[test]
    fn all_adapter_config_pipeline_variants_answer() {
        assert_answers_three_questions(
            &AdapterError::Unsupported {
                operation: "Restart".into(),
                service_type: "static".into(),
            }
            .user_message(),
        );
        assert_answers_three_questions(
            &ConfigError::SchemaTooNew {
                found: 9,
                supported: 1,
            }
            .user_message(),
        );
        assert_answers_three_questions(
            &PipelineError::Cancelled {
                step: "Build".into(),
                mid_mutation: true,
            }
            .user_message(),
        );
    }
}
