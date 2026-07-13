use serde::Deserialize;

use crate::adapters::Capabilities;
use crate::config::SiteStatus;
use crate::error::AdapterError;
use crate::ssh::RemoteCommand;

pub fn capabilities() -> Capabilities {
    Capabilities {
        can_start_stop: true,
        can_restart: true,
        has_logs: true,
        status_is_reliable: true,
    }
}

pub fn status_command() -> RemoteCommand {
    RemoteCommand::literal("pm2 jlist")
}

pub fn start_command(app: &str) -> RemoteCommand {
    RemoteCommand::new("pm2 start {}", vec![app.to_string()])
}

pub fn stop_command(app: &str) -> RemoteCommand {
    RemoteCommand::new("pm2 stop {}", vec![app.to_string()])
}

pub fn restart_command(app: &str) -> RemoteCommand {
    RemoteCommand::new("pm2 restart {}", vec![app.to_string()])
}

pub fn logs_command(app: &str) -> RemoteCommand {
    RemoteCommand::new("pm2 logs {} --lines 200", vec![app.to_string()])
}

/// A one-shot log tail that returns instead of following.
pub fn logs_snapshot_command(app: &str) -> RemoteCommand {
    RemoteCommand::new("pm2 logs {} --lines 200 --nostream", vec![app.to_string()])
}

#[derive(Debug, Deserialize)]
struct Pm2Entry {
    name: String,
    #[serde(default)]
    pm2_env: Pm2Env,
}

#[derive(Debug, Default, Deserialize)]
struct Pm2Env {
    #[serde(default)]
    status: String,
    #[serde(default, rename = "pm_uptime")]
    pm_uptime: Option<i64>,
}

pub fn parse_status(output: &str, app_name: &str) -> Result<SiteStatus, AdapterError> {
    let entries: Vec<Pm2Entry> =
        serde_json::from_str(output.trim()).map_err(|e| AdapterError::Unparseable {
            tool: "pm2 jlist".into(),
            detail: e.to_string(),
        })?;

    let entry = entries.iter().find(|e| e.name == app_name);
    let Some(entry) = entry else {
        return Ok(SiteStatus::Unknown {
            reason: format!("pm2 has no app named `{app_name}`"),
        });
    };

    Ok(match entry.pm2_env.status.as_str() {
        "online" => {
            let since = entry
                .pm2_env
                .pm_uptime
                .and_then(chrono::DateTime::from_timestamp_millis);
            SiteStatus::Running { since }
        }
        "stopped" => SiteStatus::Stopped,
        "errored" => SiteStatus::Failed {
            reason: format!("pm2 app `{app_name}` is in errored state"),
        },
        "launching" | "one-launch-status" => SiteStatus::Checking,
        other => SiteStatus::Unknown {
            reason: format!("pm2 reported status `{other}` for `{app_name}`"),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const JLIST: &str = r#"[
      {"name":"api","pm2_env":{"status":"online","pm_uptime":1752242602000}},
      {"name":"worker","pm2_env":{"status":"stopped"}},
      {"name":"broken","pm2_env":{"status":"errored"}}
    ]"#;

    #[test]
    fn online_app_is_running_with_since() {
        match parse_status(JLIST, "api").unwrap() {
            SiteStatus::Running { since } => assert!(since.is_some()),
            other => panic!("expected Running, got {other:?}"),
        }
    }

    #[test]
    fn stopped_app_is_stopped() {
        assert_eq!(parse_status(JLIST, "worker").unwrap(), SiteStatus::Stopped);
    }

    #[test]
    fn errored_app_is_failed() {
        assert!(matches!(
            parse_status(JLIST, "broken").unwrap(),
            SiteStatus::Failed { .. }
        ));
    }

    #[test]
    fn missing_app_is_unknown_not_a_lie() {
        match parse_status(JLIST, "ghost").unwrap() {
            SiteStatus::Unknown { reason } => assert!(reason.contains("ghost")),
            other => panic!("expected Unknown, got {other:?}"),
        }
    }

    #[test]
    fn garbage_is_a_parse_error() {
        assert!(matches!(
            parse_status("not json", "api").unwrap_err(),
            AdapterError::Unparseable { .. }
        ));
    }

    #[test]
    fn restart_escapes_app_name() {
        assert_eq!(
            restart_command("app; reboot").render(),
            "pm2 restart 'app; reboot'"
        );
    }
}
