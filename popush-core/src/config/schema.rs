//! Serde types for `config.toml` and the core domain types (§7.2, §7.3).
//!
//! These are the types that cross the IPC boundary; `ts-rs` generates their
//! TypeScript twins into `src/types/generated.ts` (Resolved Decision: ts-rs).
//! Hand-editing the generated file is forbidden — it is build output.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ids::{ServerId, SiteId};

/// The current config schema version. Bumped only when the on-disk shape changes;
/// the loader migrates older versions forward (§7.2).
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// The whole config file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct Config {
    /// Schema version for forward-migration. Absent in v1 files means version 1.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// User preferences.
    #[serde(default)]
    pub preferences: Preferences,
    /// Configured servers. Renamed to `server` in TOML for `[[server]]` tables.
    #[serde(rename = "server", default)]
    pub servers: Vec<ServerConfig>,
}

fn default_schema_version() -> u32 {
    1
}

/// User preferences block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct Preferences {
    /// Theme selection.
    #[serde(default)]
    pub theme: Theme,
    /// Accent colour key. Only "violet" is defined in v1 (Resolved Decision).
    #[serde(default = "default_accent")]
    pub accent: String,
    /// Background poll interval in seconds; 0 disables polling (§6.4).
    #[serde(default = "default_poll_interval")]
    pub poll_interval_seconds: u64,
    /// Whether destructive actions (Stop) ask for confirmation.
    #[serde(default = "default_true")]
    pub confirm_destructive: bool,
    /// Default branch used when a site does not override it.
    #[serde(default = "default_branch")]
    pub default_branch: String,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            accent: default_accent(),
            poll_interval_seconds: default_poll_interval(),
            confirm_destructive: true,
            default_branch: default_branch(),
        }
    }
}

fn default_accent() -> String {
    "violet".into()
}
fn default_poll_interval() -> u64 {
    60
}
fn default_true() -> bool {
    true
}
fn default_branch() -> String {
    "main".into()
}

/// Theme selection; follows the system by default (§14.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Follow the OS / KDE Plasma preference.
    #[default]
    System,
    /// Force dark.
    Dark,
    /// Force light.
    Light,
}

/// A configured server. `identity_file` is a **path only** (D7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ServerConfig {
    /// Stable id, referenced by sites and the UI.
    pub id: ServerId,
    /// Human label shown in the sidebar.
    pub label: String,
    /// Hostname or IP.
    pub host: String,
    /// SSH port.
    #[serde(default = "default_port")]
    pub port: u16,
    /// SSH username.
    pub username: String,
    /// Path to the private key. **Never the key itself** (D7).
    pub identity_file: PathBuf,
    /// Optional jump host.
    #[serde(default)]
    pub proxy_jump: Option<String>,
    /// Sites hosted on this server. `[[server.site]]` in TOML.
    #[serde(rename = "site", default)]
    pub sites: Vec<SiteConfig>,
}

fn default_port() -> u16 {
    22
}

/// A configured site (one service on a server).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SiteConfig {
    /// Stable id.
    pub id: SiteId,
    /// Human label.
    pub label: String,
    /// Absolute path on the server where the repo/site lives.
    pub remote_path: PathBuf,
    /// How the service runs; selects the adapter.
    pub service_type: ServiceKind,
    /// Compose project, unit name, or pm2 app name (unused for static).
    #[serde(default)]
    pub service_name: Option<String>,
    /// Web root for static sites; defaults to `remote_path` if absent.
    #[serde(default)]
    pub web_root: Option<PathBuf>,
    /// Optional build command run on the server during Ship It.
    #[serde(default)]
    pub build_command: Option<String>,
    /// Git remote name (e.g. `origin`).
    #[serde(default = "default_remote")]
    pub git_remote: String,
    /// Git branch to deploy.
    #[serde(default = "default_branch")]
    pub git_branch: String,
    /// Local clone path for the git panel.
    #[serde(default)]
    pub local_path: Option<PathBuf>,
    /// Public URL, shown in the UI.
    #[serde(default)]
    pub live_url: Option<String>,
    /// Optional health check URL; presence upgrades static status honesty (§9.5).
    #[serde(default)]
    pub health_check_url: Option<String>,
}

fn default_remote() -> String {
    "origin".into()
}

/// The service kind as written in config; selects an adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum ServiceKind {
    /// Docker Compose project.
    Docker,
    /// systemd unit.
    Systemd,
    /// pm2 application.
    Pm2,
    /// Static files served by a web server.
    Static,
}

/// The resolved service configuration, richer than [`ServiceKind`] (§7.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServiceConfig {
    /// Docker Compose.
    Docker {
        /// Compose project name.
        compose_project: String,
        /// Optional explicit compose file.
        compose_file: Option<PathBuf>,
    },
    /// systemd unit.
    Systemd {
        /// Unit name.
        unit: String,
    },
    /// pm2 app.
    Pm2 {
        /// App name.
        app_name: String,
    },
    /// Static files.
    Static {
        /// Web root directory.
        web_root: PathBuf,
    },
}

/// Live status of a site (§7.3). The static adapter defaults to `Unknown` (D12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SiteStatus {
    /// Running, optionally since a known time.
    Running {
        /// When it started, if the service reports it.
        since: Option<DateTime<Utc>>,
    },
    /// Cleanly stopped.
    Stopped,
    /// Failed, with a reason describing which container/unit is down.
    Failed {
        /// Human reason.
        reason: String,
    },
    /// Genuinely unknown (server unreachable, or static without a health check).
    Unknown {
        /// Why it is unknown, shown in the tooltip (§9.5).
        reason: String,
    },
    /// A check is in flight.
    Checking,
}

/// Local git status (§7.3), produced by the git subsystem, rendered by the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct GitStatus {
    /// Current branch name.
    pub branch: String,
    /// Commits ahead of the tracking remote.
    pub ahead: usize,
    /// Commits behind the tracking remote.
    pub behind: usize,
    /// Changed files.
    pub changed_files: Vec<ChangedFile>,
    /// Whether the working tree has unresolved conflicts.
    pub has_conflicts: bool,
    /// The remote URL.
    pub remote_url: String,
    /// Whether the remote is SSH (true) or HTTPS (false); drives the wizard.
    pub remote_is_ssh: bool,
}

/// One changed file in the working tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ChangedFile {
    /// Repo-relative path.
    pub path: PathBuf,
    /// The kind of change.
    pub change: ChangeKind,
    /// Whether the change is staged.
    pub staged: bool,
}

/// The kind of change to a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind {
    /// Newly added and staged.
    Added,
    /// Modified.
    Modified,
    /// Deleted.
    Deleted,
    /// Renamed.
    Renamed,
    /// Present but untracked.
    Untracked,
}

impl SiteConfig {
    /// Resolve the rich [`ServiceConfig`] from the flat TOML fields, applying the
    /// documented defaults (service_name defaults to id, web_root to remote_path).
    /// Returns the field name and problem if a required field is missing, so the
    /// loader can produce a message that names the field (§7.2 gate).
    pub fn resolve_service(&self) -> Result<ServiceConfig, (&'static str, String)> {
        match self.service_type {
            ServiceKind::Docker => Ok(ServiceConfig::Docker {
                compose_project: self
                    .service_name
                    .clone()
                    .unwrap_or_else(|| self.id.0.clone()),
                compose_file: None,
            }),
            ServiceKind::Systemd => Ok(ServiceConfig::Systemd {
                unit: self
                    .service_name
                    .clone()
                    .ok_or(("service_name", "systemd sites require a unit name".into()))?,
            }),
            ServiceKind::Pm2 => Ok(ServiceConfig::Pm2 {
                app_name: self
                    .service_name
                    .clone()
                    .ok_or(("service_name", "pm2 sites require an app name".into()))?,
            }),
            ServiceKind::Static => Ok(ServiceConfig::Static {
                web_root: self
                    .web_root
                    .clone()
                    .unwrap_or_else(|| self.remote_path.clone()),
            }),
        }
    }
}
