use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Check {
    LocalKeyExists,
    KeyInAgent,
    KeyOnGithub,
    LocalRemoteIsSsh,
    TestPush,
    ServerCanPull,
    ServerRemoteIsSsh,
}

impl Check {
    pub const ALL: [Check; 7] = [
        Check::LocalKeyExists,
        Check::KeyInAgent,
        Check::KeyOnGithub,
        Check::LocalRemoteIsSsh,
        Check::TestPush,
        Check::ServerCanPull,
        Check::ServerRemoteIsSsh,
    ];

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CheckStatus {
    Pass,
    Fail { what_is_wrong: String },
    NotApplicable { why: String },
    Running,
}
