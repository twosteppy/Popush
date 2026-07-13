//! Load, validate, and migrate config. Rejects malformed config with a
//! message that names the field and the problem (Phase 2 gate).

use std::collections::HashSet;

use crate::config::schema::{Config, ServiceKind, CURRENT_SCHEMA_VERSION};
use crate::error::ConfigError;

/// Parse and validate config from a TOML string. Kept string-based (rather than
/// path-based) so it is testable without touching the filesystem;
/// the binary layer reads the file and calls this.
pub fn load_from_str(text: &str) -> Result<Config, ConfigError> {
    let config: Config = toml::from_str(text).map_err(|e| ConfigError::Malformed {
        detail: e.to_string(),
    })?;
    migrate(config).and_then(|c| {
        validate(&c)?;
        Ok(c)
    })
}

/// Migrate an older-schema config forward. v1 is the first version, so migration
/// is currently identity plus a guard against configs from the future.
pub fn migrate(config: Config) -> Result<Config, ConfigError> {
    if config.schema_version > CURRENT_SCHEMA_VERSION {
        return Err(ConfigError::SchemaTooNew {
            found: config.schema_version,
            supported: CURRENT_SCHEMA_VERSION,
        });
    }
    // Future migrations (v1 -> v2, ...) chain here.
    Ok(config)
}

/// Reject a NUL byte in a config value. A NUL truncates the C string sshd hands
/// to the remote shell, silently cutting the command in half.
fn reject_nul(field: String, value: &str) -> Result<(), ConfigError> {
    if value.contains('\0') {
        return Err(ConfigError::InvalidField {
            field,
            problem: "must not contain a NUL byte".into(),
        });
    }
    Ok(())
}

/// Validate a value that becomes an operand of a remote command (a path, a unit,
/// an app name). Rejects a NUL and a leading `-`: even though the value is always
/// shell-escaped (so it cannot break quoting), a leading `-` would still be read
/// by the target tool (`systemctl`, `pm2`, `ls`, `cd`) as an *option* rather than
/// the intended operand.
fn check_operand(field: String, value: &str) -> Result<(), ConfigError> {
    reject_nul(field.clone(), value)?;
    if value.starts_with('-') {
        return Err(ConfigError::InvalidField {
            field,
            problem: "must not start with '-' (it would be read as a command option)".into(),
        });
    }
    Ok(())
}

/// Validate a parsed config. Every failure names the offending field.
pub fn validate(config: &Config) -> Result<(), ConfigError> {
    if config.preferences.poll_interval_seconds > 24 * 60 * 60 {
        return Err(ConfigError::InvalidField {
            field: "preferences.poll_interval_seconds".into(),
            problem: "must be 0 (disabled) or a sane number of seconds".into(),
        });
    }

    let mut server_ids = HashSet::new();
    let mut site_ids = HashSet::new();

    for server in &config.servers {
        if server.id.0.trim().is_empty() {
            return Err(ConfigError::InvalidField {
                field: "server.id".into(),
                problem: "server id must not be empty".into(),
            });
        }
        if !server_ids.insert(server.id.0.clone()) {
            return Err(ConfigError::InvalidField {
                field: "server.id".into(),
                problem: format!("duplicate server id `{}`", server.id.0),
            });
        }
        if server.host.trim().is_empty() {
            return Err(ConfigError::InvalidField {
                field: format!("server[{}].host", server.id.0),
                problem: "host must not be empty".into(),
            });
        }
        if server.username.trim().is_empty() {
            return Err(ConfigError::InvalidField {
                field: format!("server[{}].username", server.id.0),
                problem: "username must not be empty".into(),
            });
        }
        if server.port == 0 {
            return Err(ConfigError::InvalidField {
                field: format!("server[{}].port", server.id.0),
                problem: "port must be between 1 and 65535".into(),
            });
        }
        reject_nul(format!("server[{}].host", server.id.0), &server.host)?;
        reject_nul(
            format!("server[{}].username", server.id.0),
            &server.username,
        )?;
        // guardrail: reject anything that looks like an inline private key,
        // so a mis-paste never silently lands a secret in config.toml.
        let idf = server.identity_file.to_string_lossy();
        if idf.contains("PRIVATE KEY") || idf.contains("BEGIN OPENSSH") {
            return Err(ConfigError::InvalidField {
                field: format!("server[{}].identity_file", server.id.0),
                problem: "must be a path to a key, never the key itself".into(),
            });
        }

        for site in &server.sites {
            if site.id.0.trim().is_empty() {
                return Err(ConfigError::InvalidField {
                    field: "site.id".into(),
                    problem: "site id must not be empty".into(),
                });
            }
            if !site_ids.insert(site.id.0.clone()) {
                return Err(ConfigError::InvalidField {
                    field: "site.id".into(),
                    problem: format!("duplicate site id `{}`", site.id.0),
                });
            }
            if site.remote_path.as_os_str().is_empty() {
                return Err(ConfigError::InvalidField {
                    field: format!("site[{}].remote_path", site.id.0),
                    problem: "remote_path must not be empty".into(),
                });
            }
            // Fields that become remote-command operands must be shell-safe.
            check_operand(
                format!("site[{}].remote_path", site.id.0),
                &site.remote_path.to_string_lossy(),
            )?;
            if let Some(root) = &site.web_root {
                check_operand(
                    format!("site[{}].web_root", site.id.0),
                    &root.to_string_lossy(),
                )?;
            }
            if let Some(name) = &site.service_name {
                check_operand(format!("site[{}].service_name", site.id.0), name)?;
            }
            if let Some(build) = &site.build_command {
                reject_nul(format!("site[{}].build_command", site.id.0), build)?;
            }
            // A build command is required to make sense of the Build step, but
            // only for non-static services where a build is expected.
            if matches!(
                site.service_type,
                ServiceKind::Systemd | ServiceKind::Pm2 | ServiceKind::Docker
            ) {
                // resolve_service surfaces missing service_name with a named field.
                site.resolve_service()
                    .map_err(|(field, problem)| ConfigError::InvalidField {
                        field: format!("site[{}].{}", site.id.0, field),
                        problem,
                    })?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"
[preferences]
theme = "dark"
poll_interval_seconds = 60

[[server]]
id = "vps-main"
label = "Main VPS"
host = "203.0.113.10"
port = 22
username = "deploy"
identity_file = "~/.ssh/id_ed25519"

  [[server.site]]
  id = "sterling"
  label = "Sterling"
  remote_path = "/srv/sterling"
  service_type = "docker"
  service_name = "sterling"
  build_command = "pnpm build"
  local_path = "~/dev/sterling"
"#;

    #[test]
    fn good_config_loads_and_round_trips() {
        let cfg = load_from_str(GOOD).expect("should load");
        assert_eq!(cfg.servers.len(), 1);
        assert_eq!(cfg.servers[0].sites.len(), 1);
        // Round-trip: serialising and reloading yields the same structure.
        let text = toml::to_string(&cfg).unwrap();
        let reloaded = load_from_str(&text).unwrap();
        assert_eq!(cfg, reloaded);
    }

    #[test]
    fn missing_schema_version_defaults_to_one() {
        let cfg = load_from_str(GOOD).unwrap();
        assert_eq!(cfg.schema_version, 1);
    }

    #[test]
    fn malformed_toml_is_rejected() {
        let err = load_from_str("this is not = = toml").unwrap_err();
        assert!(matches!(err, ConfigError::Malformed { .. }));
    }

    #[test]
    fn duplicate_server_id_names_the_field() {
        let text = r#"
[[server]]
id = "a"
label = "A"
host = "h"
username = "u"
identity_file = "~/.ssh/k"
[[server]]
id = "a"
label = "B"
host = "h2"
username = "u"
identity_file = "~/.ssh/k"
"#;
        let err = load_from_str(text).unwrap_err();
        match err {
            ConfigError::InvalidField { field, problem } => {
                assert_eq!(field, "server.id");
                assert!(problem.contains("duplicate"));
            }
            other => panic!("wrong error: {other:?}"),
        }
    }

    #[test]
    fn systemd_site_without_unit_names_the_field() {
        let text = r#"
[[server]]
id = "a"
label = "A"
host = "h"
username = "u"
identity_file = "~/.ssh/k"
  [[server.site]]
  id = "s"
  label = "S"
  remote_path = "/srv/s"
  service_type = "systemd"
"#;
        let err = load_from_str(text).unwrap_err();
        match err {
            ConfigError::InvalidField { field, problem } => {
                assert!(field.contains("service_name"), "field was {field}");
                assert!(problem.contains("unit"));
            }
            other => panic!("wrong error: {other:?}"),
        }
    }

    #[test]
    fn inline_private_key_in_identity_file_is_rejected() {
        let text = r#"
[[server]]
id = "a"
label = "A"
host = "h"
username = "u"
identity_file = "-----BEGIN OPENSSH PRIVATE KEY-----"
"#;
        let err = load_from_str(text).unwrap_err();
        match err {
            ConfigError::InvalidField { field, .. } => assert!(field.contains("identity_file")),
            other => panic!("wrong error: {other:?}"),
        }
    }

    #[test]
    fn config_from_the_future_is_rejected() {
        let text = "schema_version = 999\n";
        let err = load_from_str(text).unwrap_err();
        assert!(matches!(err, ConfigError::SchemaTooNew { found: 999, .. }));
    }

    #[test]
    fn leading_dash_remote_path_is_rejected() {
        let text = r#"
[[server]]
id = "a"
label = "A"
host = "h"
username = "u"
identity_file = "~/.ssh/k"
  [[server.site]]
  id = "s"
  label = "S"
  remote_path = "--output=/etc/x"
  service_type = "static"
"#;
        match load_from_str(text).unwrap_err() {
            ConfigError::InvalidField { field, problem } => {
                assert!(field.contains("remote_path"), "field was {field}");
                assert!(problem.contains("option"));
            }
            other => panic!("wrong error: {other:?}"),
        }
    }

    #[test]
    fn leading_dash_service_name_is_rejected() {
        let text = r#"
[[server]]
id = "a"
label = "A"
host = "h"
username = "u"
identity_file = "~/.ssh/k"
  [[server.site]]
  id = "s"
  label = "S"
  remote_path = "/srv/s"
  service_type = "systemd"
  service_name = "--version"
"#;
        match load_from_str(text).unwrap_err() {
            ConfigError::InvalidField { field, .. } => {
                assert!(field.contains("service_name"), "field was {field}");
            }
            other => panic!("wrong error: {other:?}"),
        }
    }

    #[test]
    fn nul_byte_in_remote_path_is_rejected() {
        // A literal NUL cannot appear in a TOML string, so exercise `validate`
        // directly the way the in-app add-server flow does: build a config, then
        // inject a NUL into a path and confirm validation refuses it.
        let mut cfg = load_from_str(GOOD).expect("base config");
        cfg.servers[0].sites[0].remote_path = std::path::PathBuf::from("/srv/\0x");
        match validate(&cfg).unwrap_err() {
            ConfigError::InvalidField { field, problem } => {
                assert!(field.contains("remote_path"));
                assert!(problem.contains("NUL"));
            }
            other => panic!("wrong error: {other:?}"),
        }
    }
}
