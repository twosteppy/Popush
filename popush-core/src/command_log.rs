use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ids::ServerId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct CommandOutcome {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub command_display: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct CommandLogEntry {
    pub timestamp: DateTime<Utc>,
    pub server: ServerId,
    pub command: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
}

impl CommandLogEntry {
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
