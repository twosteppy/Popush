use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ids::{ServerId, SiteId};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct Config {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub preferences: Preferences,
    #[serde(rename = "server", default)]
    pub servers: Vec<ServerConfig>,
}

fn default_schema_version() -> u32 {
    1
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            preferences: Preferences::default(),
            servers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct Preferences {
    #[serde(default)]
    pub theme: Theme,
    #[serde(default = "default_accent")]
    pub accent: String,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_seconds: u64,
    #[serde(default = "default_true")]
    pub confirm_destructive: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Dark,
    Light,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ServerConfig {
    pub id: ServerId,
    pub label: String,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub username: String,
    pub identity_file: PathBuf,
    #[serde(default)]
    pub proxy_jump: Option<String>,
    #[serde(rename = "site", default)]
    pub sites: Vec<SiteConfig>,
}

fn default_port() -> u16 {
    22
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SiteConfig {
    pub id: SiteId,
    pub label: String,
    pub remote_path: PathBuf,
    pub service_type: ServiceKind,
    #[serde(default)]
    pub service_name: Option<String>,
    #[serde(default)]
    pub web_root: Option<PathBuf>,
    #[serde(default)]
    pub build_command: Option<String>,
    #[serde(default = "default_remote")]
    pub git_remote: String,
    #[serde(default = "default_branch")]
    pub git_branch: String,
    #[serde(default)]
    pub local_path: Option<PathBuf>,
    #[serde(default)]
    pub live_url: Option<String>,
    #[serde(default)]
    pub health_check_url: Option<String>,
}

fn default_remote() -> String {
    "origin".into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum ServiceKind {
    Docker,
    Systemd,
    Pm2,
    Static,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServiceConfig {
    Docker {
        compose_project: String,
        compose_file: Option<PathBuf>,
    },
    Systemd {
        unit: String,
    },
    Pm2 {
        app_name: String,
    },
    Static {
        web_root: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SiteStatus {
    Running { since: Option<DateTime<Utc>> },
    Stopped,
    Failed { reason: String },
    Unknown { reason: String },
    Checking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct GitStatus {
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub changed_files: Vec<ChangedFile>,
    pub has_conflicts: bool,
    pub remote_url: String,
    pub remote_is_ssh: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ChangedFile {
    pub path: PathBuf,
    pub change: ChangeKind,
    pub staged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Untracked,
}

impl SiteConfig {
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
