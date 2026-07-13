//! The wizard checks C1–C7. Each is independent and resolves to pass,
//! fail, or not-applicable.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The seven checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Check {
    /// C1: an SSH key exists locally.
    LocalKeyExists,
    /// C2: the key is loaded in ssh-agent.
    KeyInAgent,
    /// C3: the key is registered with GitHub.
    KeyOnGithub,
    /// C4: the local remote is SSH, not HTTPS.
    LocalRemoteIsSsh,
    /// C5: a test push works.
    TestPush,
    /// C6: the VPS can pull from GitHub (the deploy-key step).
    ServerCanPull,
    /// C7: the VPS git remote is SSH.
    ServerRemoteIsSsh,
}

impl Check {
    /// All checks in the order the wizard presents them.
    pub const ALL: [Check; 7] = [
        Check::LocalKeyExists,
        Check::KeyInAgent,
        Check::KeyOnGithub,
        Check::LocalRemoteIsSsh,
        Check::TestPush,
        Check::ServerCanPull,
        Check::ServerRemoteIsSsh,
    ];

    /// The plain-English name shown on the checklist row.
    pub fn title(self) -> &'static str {
        match self {
            Check::LocalKeyExists => "An SSH key exists on this machine",
            Check::KeyInAgent => "Your key is loaded in the SSH agent",
            Check::KeyOnGithub => "Your key is registered with GitHub",
            Check::LocalRemoteIsSsh => "This repo uses an SSH remote",
            Check::TestPush => "A test push works",
            Check::ServerCanPull => "The server can pull from GitHub",
            Check::ServerRemoteIsSsh => "The server's remote uses SSH",
        }
    }

    /// The one-line explanation shown under the title.
    pub fn explanation(self) -> &'static str {
        match self {
            Check::LocalKeyExists => "Popush needs a key to talk to your server and GitHub.",
            Check::KeyInAgent => "The agent holds your key so Popush never sees your passphrase.",
            Check::KeyOnGithub => "GitHub must recognise your key to accept pushes.",
            Check::LocalRemoteIsSsh => "HTTPS remotes need a token; SSH does not.",
            Check::TestPush => "Confirms end to end that you can push.",
            Check::ServerCanPull => "The server pulls your code during a deploy.",
            Check::ServerRemoteIsSsh => "Same reason as your local remote.",
        }
    }
}

/// The resolved status of a check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CheckStatus {
    /// The check passed; nothing to do.
    Pass,
    /// The check failed; a fix is offered.
    Fail {
        /// What is wrong, shown on the expanded row.
        what_is_wrong: String,
    },
    /// The check does not apply (e.g. server checks with no server yet).
    NotApplicable {
        /// Why it does not apply.
        why: String,
    },
    /// The check is currently running.
    Running,
}
