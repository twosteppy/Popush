//! The command log: a permanent, inspectable record of every remote
//! command Popush ran, so the user can always answer "what did this app actually
//! do to my server?" The log holds only what is safe to show, the command as
//! sent (already escaped), never key material.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ids::ServerId;

/// The outcome of running a remote command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct CommandOutcome {
    /// The process exit code.
    pub exit_code: i32,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
    /// How long it took, in milliseconds.
    pub duration_ms: u64,
    /// The exact command sent, safe to display in the log.
    pub command_display: String,
}

/// One entry in the command log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct CommandLogEntry {
    /// When the command was sent.
    pub timestamp: DateTime<Utc>,
    /// Which server it went to.
    pub server: ServerId,
    /// The exact command sent.
    pub command: String,
    /// The exit code, or `None` if the command did not complete.
    pub exit_code: Option<i32>,
    /// Duration in milliseconds.
    pub duration_ms: u64,
}

impl CommandLogEntry {
    /// Record a completed command from its outcome.
    pub fn from_outcome(
        timestamp: DateTime<Utc>,
        server: ServerId,
        outcome: &CommandOutcome,
    ) -> Self {
        Self {
            timestamp,
            server,
            command: outcome.command_display.clone(),
            exit_code: Some(outcome.exit_code),
            duration_ms: outcome.duration_ms,
        }
    }
}
