//! The error taxonomy.
//!
//! Prime Directive: **no generic errors.** Every failure names its step,
//! states its consequence for the user, and offers a next action. The strings
//! "Deploy failed" and "Something went wrong" are banned and asserted absent by
//! [`crate::pipeline::messages`] tests and the banned-strings test.
//!
//! Errors are *structured*: each variant carries the context a good
//! message needs. A `String` error cannot answer the three questions
//! because by the time it reaches the UI the structure is gone. So every variant
//! here can produce a [`UserMessage`] with all three parts filled in.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A user-facing message that answers the three questions Rendered by
/// the frontend; never assembled there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct UserMessage {
    /// What happened. Specific, the failing step, never "Deploy failed".
    pub headline: String,
    /// What it means for the user. E.g. "Your site is still running the previous
    /// version."
    pub consequence: String,
    /// What to do now: an action, a command, or a button the UI can offer.
    pub next_action: NextAction,
}

/// The concrete "what do I do now" affordance the UI should present.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NextAction {
    /// Show a shell command with a copy button.
    RunCommand {
        /// The exact command to display and copy.
        command: String,
    },
    /// Offer a button that triggers an in-app flow (e.g. the setup wizard).
    OpenFlow {
        /// The flow identifier (e.g. `"wizard"`).
        flow: String,
        /// The button label.
        label: String,
    },
    /// Offer a plain retry button.
    Retry,
    /// Nothing to do but read the guidance; the text itself is the action.
    Advice {
        /// The guidance text.
        text: String,
    },
}

/// Top-level error crossing the IPC boundary. Each subsystem has its own typed
/// enum; `thiserror` gives them `Display` and `Error` without `Box<dyn Error>`.
#[derive(Debug, thiserror::Error, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum AppError {
    /// An SSH-layer failure.
    #[error("ssh: {0}")]
    Ssh(SshError),
    /// A local git failure.
    #[error("git: {0}")]
    Git(GitError),
    /// A service-adapter failure.
    #[error("adapter: {0}")]
    Adapter(AdapterError),
    /// A config load/validate/migrate failure.
    #[error("config: {0}")]
    Config(ConfigError),
    /// A pipeline (Ship It) failure.
    #[error("pipeline: {0}")]
    Pipeline(PipelineError),
}

/// Reasons authentication can fail, kept distinct so the message can be exact.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize, TS)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum AuthFailureReason {
    /// The agent rejected every identity it holds.
    #[error("no identity in ssh-agent was accepted by the server")]
    AgentRejected,
    /// `SSH_AUTH_SOCK` was not set, so there is no agent to talk to.
    #[error("SSH_AUTH_SOCK is not set; no ssh-agent is running")]
    NoAgentSocket,
    /// The server accepted no offered method.
    #[error("the server accepted none of the offered authentication methods")]
    AllMethodsExhausted,
}

/// SSH-layer errors. Every variant carries the context a message needs.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize, TS)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum SshError {
    /// The host could not be reached at all.
    #[error("host {host} is unreachable: {detail}")]
    HostUnreachable {
        /// The host that could not be reached.
        host: String,
        /// The underlying OS/network detail.
        detail: String,
    },
    /// Authentication was refused.
    #[error("authentication failed: {reason}")]
    AuthFailed {
        /// The specific reason auth failed.
        reason: AuthFailureReason,
    },
    /// The stored host key does not match, a possible man-in-the-middle.
    #[error("host key mismatch for {host}")]
    HostKeyMismatch {
        /// The host whose key changed.
        host: String,
        /// The pinned key Popush expected.
        expected: String,
        /// The key the server actually presented.
        got: String,
    },
    /// The host is not yet known; the user must verify the fingerprint.
    #[error("host {host} is unknown; fingerprint {fingerprint}")]
    HostKeyUnknown {
        /// The unknown host.
        host: String,
        /// The fingerprint for the user to verify.
        fingerprint: String,
    },
    /// The configured key is passphrase-protected and not loaded in the agent.
    #[error("key {path} has a passphrase and is not in the agent")]
    KeyNotInAgent {
        /// Path to the key that is not loaded.
        path: PathBuf,
    },
    /// The configured key file does not exist.
    #[error("key file {path} not found")]
    KeyNotFound {
        /// Path that does not exist.
        path: PathBuf,
    },
    /// A remote command exited non-zero.
    #[error("command exited {exit_code}")]
    CommandFailed {
        /// The command that was run (safe display form).
        command: String,
        /// The non-zero exit code.
        exit_code: i32,
        /// Captured stderr.
        stderr: String,
    },
    /// The connection dropped mid-operation.
    #[error("connection lost")]
    ConnectionLost,
    /// An operation timed out, after the given number of milliseconds.
    #[error("timed out after {after_ms}ms")]
    Timeout {
        /// How long Popush waited, in milliseconds.
        after_ms: u64,
    },
}

/// Local git errors. The refusal messages are verbatim from the spec.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize, TS)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum GitError {
    /// Unresolved merge conflicts; Popush refuses to proceed.
    #[error("{count} files have unresolved merge conflicts")]
    MergeConflicts {
        /// Number of conflicted files.
        count: usize,
        /// The conflicted paths.
        files: Vec<PathBuf>,
    },
    /// Not on a branch.
    #[error("detached HEAD")]
    DetachedHead,
    /// The current branch has no upstream configured.
    #[error("branch {branch} has no upstream")]
    NoUpstream {
        /// The branch with no upstream.
        branch: String,
    },
    /// The remote is HTTPS, which needs a token; route to the wizard.
    #[error("remote {url} is HTTPS, not SSH")]
    HttpsRemote {
        /// The HTTPS remote URL.
        url: String,
    },
    /// The remote is neither SSH nor HTTPS (e.g. `git://`, a local path, or an
    /// `ext::` helper). Popush only ever pushes over SSH, so anything else is
    /// refused rather than trusted.
    #[error("remote {url} is not an SSH remote")]
    NonSshRemote {
        /// The rejected remote URL.
        url: String,
    },
    /// A push was rejected as non-fast-forward.
    #[error("push rejected: non-fast-forward")]
    PushRejectedNonFastForward,
    /// A push was rejected for lack of permission.
    #[error("push rejected: permission denied")]
    PushRejectedPermission,
    /// libgit2 reported an error not otherwise classified.
    #[error("git operation failed: {detail}")]
    Operation {
        /// The underlying libgit2 detail.
        detail: String,
    },
}

/// Service-adapter errors.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize, TS)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum AdapterError {
    /// The status output could not be parsed into a known shape.
    #[error("could not parse {tool} output: {detail}")]
    Unparseable {
        /// The tool whose output failed to parse (e.g. `docker compose ps`).
        tool: String,
        /// The parse detail.
        detail: String,
    },
    /// The operation is not supported by this adapter (e.g. restart on static).
    #[error("{operation} is not supported for {service_type} sites")]
    Unsupported {
        /// The unsupported operation.
        operation: String,
        /// The service type that lacks it.
        service_type: String,
    },
    /// The underlying SSH command failed.
    #[error("remote command failed: {0}")]
    Ssh(SshError),
}

/// Config load/validate/migrate errors.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize, TS)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ConfigError {
    /// The file could not be read.
    #[error("cannot read config at {path}: {detail}")]
    Unreadable {
        /// The config path.
        path: PathBuf,
        /// The OS read error detail.
        detail: String,
    },
    /// The TOML did not parse.
    #[error("config is not valid TOML: {detail}")]
    Malformed {
        /// The parser's message.
        detail: String,
    },
    /// A field is missing or invalid; names the field and the problem.
    #[error("field `{field}` is invalid: {problem}")]
    InvalidField {
        /// The offending field path.
        field: String,
        /// What is wrong with it.
        problem: String,
    },
    /// The config schema version is newer than this Popush understands.
    #[error("config schema version {found} is newer than supported {supported}")]
    SchemaTooNew {
        /// The version found in the file.
        found: u32,
        /// The highest version this build supports.
        supported: u32,
    },
}

/// Pipeline (Ship It) errors. The messages are the verbatim ones
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize, TS)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum PipelineError {
    /// A named step failed. Never rendered as "Deploy failed".
    #[error("step {step} failed")]
    StepFailed {
        /// The step that failed (e.g. `"Build"`).
        step: String,
        /// The failure detail.
        detail: String,
    },
    /// The run was cancelled by the user.
    #[error("cancelled at step {step}")]
    Cancelled {
        /// The step in progress when cancelled.
        step: String,
        /// Whether that step was mutating the server.
        mid_mutation: bool,
    },
}

impl AppError {
    /// Produce the user-facing message. This is the single place errors become
    /// human text, so the three-questions rule is enforced in one spot.
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
    /// Human message for an SSH error, answering all three questions.
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
    /// Human message for a git error. The refusal texts are verbatim
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
    /// Human message for an adapter error.
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
    /// Human message for a config error, always naming the field/problem.
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
    /// Human message for a pipeline error. Never "Deploy failed"; the
    /// step-specific text comes from [`crate::pipeline::messages`].
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

// Ergonomic conversions so `?` works across layers without `Box<dyn Error>`.
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

    /// Strings banned by anywhere in a user-facing message.
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
        // next_action is a non-optional enum, so "what do I do" is always present.
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
        // requires this exact guidance, with the key path and ssh-add command.
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
