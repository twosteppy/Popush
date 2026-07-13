use std::collections::HashMap;
use std::sync::Mutex;

use popush_core::command_log::CommandLogEntry;
use popush_core::config::{Config, SiteStatus};
use popush_core::ids::{PipelineId, ServerId, SiteId};

use crate::ssh::SshPool;

const MAX_COMMAND_LOG: usize = 5000;

pub struct AppState {
    inner: Mutex<Inner>,
}

struct Inner {
    config: Option<Config>,
    #[allow(dead_code)]
    connections: HashMap<ServerId, SshPool>,
    site_status: HashMap<SiteId, SiteStatus>,
    cancelled: HashMap<PipelineId, bool>,
    command_log: Vec<CommandLogEntry>,
    /// Session-only SSH passwords, keyed by server. Deliberately never
    /// persisted anywhere: closing the app forgets them.
    ssh_passwords: HashMap<ServerId, String>,
    /// If the config file exists but failed to load, the reason, so the UI can
    /// show it instead of a silent empty state.
    config_error: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                config: None,
                connections: HashMap::new(),
                site_status: HashMap::new(),
                cancelled: HashMap::new(),
                command_log: Vec::new(),
                ssh_passwords: HashMap::new(),
                config_error: None,
            }),
        }
    }

    pub fn load_config_on_startup(&self) {
        // Load any passwords the user chose to save last time. These live in a
        // separate 0600 file so they never touch the shareable config.
        let saved = load_saved_passwords();
        if !saved.is_empty() {
            self.inner.lock().unwrap().ssh_passwords.extend(saved);
        }

        let Some(path) = config_path() else {
            return;
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            return;
        };
        match popush_core::config::load_from_str(&text) {
            Ok(cfg) => {
                let mut guard = self.inner.lock().unwrap();
                guard.config = Some(cfg);
                guard.config_error = None;
            }
            Err(e) => {
                tracing::warn!(error = %e, "config failed to load");
                self.inner.lock().unwrap().config_error = Some(e.to_string());
            }
        }
    }

    pub fn config_error(&self) -> Option<String> {
        self.inner.lock().unwrap().config_error.clone()
    }

    pub fn servers(&self) -> Vec<popush_core::config::ServerConfig> {
        self.inner
            .lock()
            .unwrap()
            .config
            .as_ref()
            .map(|c| c.servers.clone())
            .unwrap_or_default()
    }

    pub fn config_snapshot(&self) -> Config {
        self.inner
            .lock()
            .unwrap()
            .config
            .clone()
            .unwrap_or_default()
    }

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

    pub fn add_site(
        &self,
        server_id: &ServerId,
        site: popush_core::config::SiteConfig,
    ) -> Result<(), popush_core::error::ConfigError> {
        let mut guard = self.inner.lock().unwrap();
        let config = guard.config.get_or_insert_with(Config::default);
        let server = config
            .servers
            .iter_mut()
            .find(|s| &s.id == server_id)
            .ok_or_else(|| popush_core::error::ConfigError::InvalidField {
                field: "server".into(),
                problem: format!("no server with id `{}`", server_id.0),
            })?;
        if let Some(existing) = server.sites.iter_mut().find(|s| s.id == site.id) {
            *existing = site;
        } else {
            server.sites.push(site);
        }
        popush_core::config::validate(config)?;
        let toml = popush_core::config::to_toml(config)?;
        drop(guard);
        write_config_file(&toml)
    }

    pub fn import_config(&self, toml: &str) -> Result<usize, popush_core::error::ConfigError> {
        let incoming = popush_core::config::load_from_str(toml)?;
        let mut guard = self.inner.lock().unwrap();
        let config = guard.config.get_or_insert_with(Config::default);
        let mut count = 0;
        for server in incoming.servers {
            popush_core::config::upsert_server(config, server);
            count += 1;
        }
        popush_core::config::validate(config)?;
        let toml_out = popush_core::config::to_toml(config)?;
        drop(guard);
        write_config_file(&toml_out)?;
        Ok(count)
    }

    /// Remember (or forget, when empty) a server's SSH password. Always held
    /// in memory for the session; when `save` is true it is also written to a
    /// private 0600 file so it survives a restart. When `save` is false any
    /// previously saved copy on disk is removed.
    pub fn set_ssh_password(&self, id: ServerId, password: String, save: bool) {
        {
            let mut guard = self.inner.lock().unwrap();
            if password.is_empty() {
                guard.ssh_passwords.remove(&id);
            } else {
                guard.ssh_passwords.insert(id.clone(), password.clone());
            }
        }
        let mut disk = load_saved_passwords();
        if save && !password.is_empty() {
            disk.insert(id, password);
        } else {
            disk.remove(&id);
        }
        save_passwords_to_disk(&disk);
    }

    /// Whether this server has a password saved on disk (so the UI can show the
    /// box pre-ticked).
    pub fn ssh_password_is_saved(&self, id: &ServerId) -> bool {
        load_saved_passwords().contains_key(id)
    }

    pub fn ssh_password(&self, id: &ServerId) -> Option<String> {
        self.inner.lock().unwrap().ssh_passwords.get(id).cloned()
    }

    pub fn remove_site(&self, id: &SiteId) -> Result<bool, popush_core::error::ConfigError> {
        let mut guard = self.inner.lock().unwrap();
        let Some(config) = guard.config.as_mut() else {
            return Ok(false);
        };
        let removed = popush_core::config::remove_site(config, id);
        let toml = popush_core::config::to_toml(config)?;
        drop(guard);
        write_config_file(&toml)?;
        Ok(removed)
    }

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

    pub fn record_command(&self, entry: CommandLogEntry) {
        let mut guard = self.inner.lock().unwrap();
        guard.command_log.push(entry);
        let len = guard.command_log.len();
        if len > MAX_COMMAND_LOG {
            guard.command_log.drain(..len - MAX_COMMAND_LOG);
        }
    }

    pub fn command_log(&self) -> Vec<CommandLogEntry> {
        self.inner.lock().unwrap().command_log.clone()
    }

    pub fn cancel(&self, id: &PipelineId) {
        self.inner
            .lock()
            .unwrap()
            .cancelled
            .insert(id.clone(), true);
    }

    pub fn is_cancelled(&self, id: &PipelineId) -> bool {
        self.inner
            .lock()
            .unwrap()
            .cancelled
            .get(id)
            .copied()
            .unwrap_or(false)
    }

    pub fn set_status(&self, site: SiteId, status: SiteStatus) {
        self.inner.lock().unwrap().site_status.insert(site, status);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn config_path() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("dev", "popush", "popush")
        .map(|d| d.config_dir().join("config.toml"))
}

fn secrets_path() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("dev", "popush", "popush")
        .map(|d| d.config_dir().join("secrets.json"))
}

/// Read saved SSH passwords, if any. A missing or unreadable file is simply an
/// empty set, never an error the user has to see.
fn load_saved_passwords() -> HashMap<ServerId, String> {
    let Some(path) = secrets_path() else {
        return HashMap::new();
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return HashMap::new();
    };
    serde_json::from_str::<HashMap<String, String>>(&text)
        .map(|m| m.into_iter().map(|(k, v)| (ServerId(k), v)).collect())
        .unwrap_or_default()
}

/// Write saved passwords to a 0600 file, or delete it when none remain.
fn save_passwords_to_disk(map: &HashMap<ServerId, String>) {
    let Some(path) = secrets_path() else {
        return;
    };
    if map.is_empty() {
        let _ = std::fs::remove_file(&path);
        return;
    }
    let plain: HashMap<&str, &str> = map
        .iter()
        .map(|(k, v)| (k.0.as_str(), v.as_str()))
        .collect();
    let Ok(json) = serde_json::to_string_pretty(&plain) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
        }
    }
    if std::fs::write(&path, json).is_ok() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
    }
}

pub fn ensure_config_file() -> Result<std::path::PathBuf, popush_core::error::ConfigError> {
    let path = config_path().ok_or_else(|| popush_core::error::ConfigError::Unreadable {
        path: "~/.config/popush/config.toml".into(),
        detail: "could not resolve the XDG config directory".into(),
    })?;
    if !path.exists() {
        let body = popush_core::config::to_toml(&Config::default())?;
        let toml = format!(
            "# Popush configuration.\n\
             # Edit this by hand, or use the in-app \"Add a server\" flow.\n\
             # No secrets live here: SSH keys come from your ssh-agent.\n\n{body}"
        );
        write_config_file(&toml)?;
    }
    Ok(path)
}

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
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
        }
    }
    std::fs::write(&path, toml).map_err(|e| popush_core::error::ConfigError::Unreadable {
        path: path.clone(),
        detail: e.to_string(),
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}
