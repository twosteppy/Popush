//! Backend-owned application state (§6.3). The frontend holds only a mirror,
//! updated by events; it never holds authoritative state.

use std::collections::HashMap;
use std::sync::Mutex;

use popush_core::command_log::CommandLogEntry;
use popush_core::config::{Config, SiteStatus};
use popush_core::ids::{PipelineId, ServerId, SiteId};

use crate::ssh::SshPool;

/// The single source of truth for the running app, guarded for concurrent access.
pub struct AppState {
    inner: Mutex<Inner>,
}

struct Inner {
    /// Loaded config, or `None` before first load / on load error.
    config: Option<Config>,
    /// Live SSH connection pools, one per server, opened lazily (§8.1). Read by
    /// the connection-pool lifecycle wiring (the remaining integration point noted
    /// in docs/DECISIONS.md); allowed dead until those command handlers land.
    #[allow(dead_code)]
    connections: HashMap<ServerId, SshPool>,
    /// Last known status per site (§6.3).
    site_status: HashMap<SiteId, SiteStatus>,
    /// In-flight pipeline cancellation flags.
    cancelled: HashMap<PipelineId, bool>,
    /// The command log (D8) — every remote command, inspectable at any time.
    command_log: Vec<CommandLogEntry>,
}

impl AppState {
    /// Create empty state. Config is loaded in `setup` (see [`crate::run`]).
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                config: None,
                connections: HashMap::new(),
                site_status: HashMap::new(),
                cancelled: HashMap::new(),
                command_log: Vec::new(),
            }),
        }
    }

    /// Load config from the XDG path on startup. A missing file is fine (D2): the
    /// app shows its empty state. A malformed file is surfaced to the UI as a
    /// config error, never a silent failure.
    pub fn load_config_on_startup(&self) {
        let Some(path) = config_path() else {
            return;
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            // No config yet — first launch. Not an error.
            return;
        };
        match popush_core::config::load_from_str(&text) {
            Ok(cfg) => {
                self.inner.lock().unwrap().config = Some(cfg);
            }
            Err(e) => {
                // Keep config None; the UI will show the config error on demand.
                tracing::warn!(error = %e, "config failed to load");
            }
        }
    }

    /// Snapshot the configured servers for the UI.
    pub fn servers(&self) -> Vec<popush_core::config::ServerConfig> {
        self.inner
            .lock()
            .unwrap()
            .config
            .as_ref()
            .map(|c| c.servers.clone())
            .unwrap_or_default()
    }

    /// Append an entry to the command log (D8).
    pub fn record_command(&self, entry: CommandLogEntry) {
        self.inner.lock().unwrap().command_log.push(entry);
    }

    /// The full command log, newest last.
    pub fn command_log(&self) -> Vec<CommandLogEntry> {
        self.inner.lock().unwrap().command_log.clone()
    }

    /// Mark a pipeline cancelled (§12.6).
    pub fn cancel(&self, id: &PipelineId) {
        self.inner
            .lock()
            .unwrap()
            .cancelled
            .insert(id.clone(), true);
    }

    /// Whether a pipeline has been cancelled.
    pub fn is_cancelled(&self, id: &PipelineId) -> bool {
        self.inner
            .lock()
            .unwrap()
            .cancelled
            .get(id)
            .copied()
            .unwrap_or(false)
    }

    /// Update the cached status of a site.
    pub fn set_status(&self, site: SiteId, status: SiteStatus) {
        self.inner.lock().unwrap().site_status.insert(site, status);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// The XDG config path, `~/.config/popush/config.toml` (§7.1).
pub fn config_path() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("dev", "popush", "popush")
        .map(|d| d.config_dir().join("config.toml"))
}
