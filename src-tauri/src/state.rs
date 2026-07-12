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
    /// The command log (D8), every remote command, inspectable at any time.
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
            // No config yet, first launch. Not an error.
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

    /// Snapshot the whole config (or a fresh default if none is loaded).
    pub fn config_snapshot(&self) -> Config {
        self.inner
            .lock()
            .unwrap()
            .config
            .clone()
            .unwrap_or_default()
    }

    /// Add or replace a server (the in-app "Add a server" flow), then persist the
    /// config to disk so the change survives a restart (§7, D6). Writing goes
    /// through `popush_core`, which keeps the file human-editable and secret-free.
    pub fn add_or_update_server(
        &self,
        server: popush_core::config::ServerConfig,
    ) -> Result<(), popush_core::error::ConfigError> {
        let mut guard = self.inner.lock().unwrap();
        let config = guard.config.get_or_insert_with(Config::default);
        popush_core::config::upsert_server(config, server);
        popush_core::config::validate(config)?;
        let toml = popush_core::config::to_toml(config)?;
        drop(guard);
        write_config_file(&toml)
    }

    /// Remove a server by id and persist.
    pub fn remove_server(&self, id: &ServerId) -> Result<bool, popush_core::error::ConfigError> {
        let mut guard = self.inner.lock().unwrap();
        let Some(config) = guard.config.as_mut() else {
            return Ok(false);
        };
        let removed = popush_core::config::remove_server(config, id);
        let toml = popush_core::config::to_toml(config)?;
        drop(guard);
        write_config_file(&toml)?;
        Ok(removed)
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

/// Write the config TOML to the XDG path, creating the directory if needed.
fn write_config_file(toml: &str) -> Result<(), popush_core::error::ConfigError> {
    let path = config_path().ok_or_else(|| popush_core::error::ConfigError::Unreadable {
        path: "~/.config/popush/config.toml".into(),
        detail: "could not resolve the XDG config directory".into(),
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            popush_core::error::ConfigError::Unreadable {
                path: parent.to_path_buf(),
                detail: e.to_string(),
            }
        })?;
    }
    std::fs::write(&path, toml).map_err(|e| popush_core::error::ConfigError::Unreadable {
        path,
        detail: e.to_string(),
    })
}
