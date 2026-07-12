//! Writing config back to disk (§7): the in-app "Add a server" and "Add a site"
//! flows persist here so a user never has to hand-edit TOML, while the file stays
//! human-editable and secret-free (D6, D7). The serialisation and the upsert/remove
//! logic are pure and tested; the binary does the actual file write.

use crate::config::schema::Config;
use crate::config::ServerConfig;
use crate::error::ConfigError;
use crate::ids::{ServerId, SiteId};

/// The header written atop a generated `config.toml`, so a user opening the file
/// sees what it is and that it holds no secrets.
const HEADER: &str = "\
# ~/.config/popush/config.toml
# Popush configuration. Safe to edit by hand.
# Contains no secrets: keys are referenced by path, never copied.
";

/// Serialise a [`Config`] to a TOML string with the reassuring header (§7.2).
pub fn to_toml(config: &Config) -> Result<String, ConfigError> {
    let body = toml::to_string_pretty(config).map_err(|e| ConfigError::Malformed {
        detail: e.to_string(),
    })?;
    Ok(format!("{HEADER}\n{body}"))
}

/// Insert a server, or replace the existing one with the same id. Returns whether
/// an existing server was replaced (vs. a fresh insert).
pub fn upsert_server(config: &mut Config, server: ServerConfig) -> bool {
    if let Some(existing) = config.servers.iter_mut().find(|s| s.id == server.id) {
        *existing = server;
        true
    } else {
        config.servers.push(server);
        false
    }
}

/// Remove a server by id. Returns whether one was removed.
pub fn remove_server(config: &mut Config, id: &ServerId) -> bool {
    let before = config.servers.len();
    config.servers.retain(|s| &s.id != id);
    config.servers.len() != before
}

/// Remove a site by id from whichever server holds it. Returns whether one was
/// removed.
pub fn remove_site(config: &mut Config, id: &SiteId) -> bool {
    let mut removed = false;
    for server in &mut config.servers {
        let before = server.sites.len();
        server.sites.retain(|s| &s.id != id);
        if server.sites.len() != before {
            removed = true;
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::loader::load_from_str;
    use crate::config::schema::ServiceKind;
    use crate::config::SiteConfig;

    fn server(id: &str) -> ServerConfig {
        ServerConfig {
            id: ServerId(id.into()),
            label: format!("Server {id}"),
            host: "203.0.113.10".into(),
            port: 22,
            username: "deploy".into(),
            identity_file: "~/.ssh/id_ed25519".into(),
            proxy_jump: None,
            sites: Vec::new(),
        }
    }

    #[test]
    fn to_toml_round_trips_and_keeps_header() {
        let mut config = Config {
            schema_version: 1,
            preferences: Default::default(),
            servers: vec![server("vps-main")],
        };
        config.servers[0].sites.push(SiteConfig {
            id: SiteId("site-a".into()),
            label: "Site A".into(),
            remote_path: "/srv/a".into(),
            service_type: ServiceKind::Docker,
            service_name: Some("a".into()),
            web_root: None,
            build_command: Some("pnpm build".into()),
            git_remote: "origin".into(),
            git_branch: "main".into(),
            local_path: Some("~/dev/a".into()),
            live_url: None,
            health_check_url: None,
        });

        let text = to_toml(&config).unwrap();
        assert!(text.contains("Safe to edit by hand"));
        // The written file must parse straight back to the same config.
        let reloaded = load_from_str(&text).unwrap();
        assert_eq!(config, reloaded);
    }

    #[test]
    fn upsert_inserts_then_replaces() {
        let mut config = Config {
            schema_version: 1,
            preferences: Default::default(),
            servers: vec![],
        };
        assert!(!upsert_server(&mut config, server("a")));
        assert_eq!(config.servers.len(), 1);
        // Same id replaces, does not duplicate.
        let mut updated = server("a");
        updated.label = "Renamed".into();
        assert!(upsert_server(&mut config, updated));
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.servers[0].label, "Renamed");
    }

    #[test]
    fn remove_server_reports_whether_it_removed() {
        let mut config = Config {
            schema_version: 1,
            preferences: Default::default(),
            servers: vec![server("a"), server("b")],
        };
        assert!(remove_server(&mut config, &ServerId("a".into())));
        assert!(!remove_server(&mut config, &ServerId("missing".into())));
        assert_eq!(config.servers.len(), 1);
    }
}
