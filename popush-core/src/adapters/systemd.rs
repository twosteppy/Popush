//! systemd adapter. Start/stop/restart need root; Popush never prompts for
//! a password, the commands assume a passwordless sudoers entry the wizard
//! *generates for the user to install by hand*, or a user unit. Popush never edits
//! sudoers itself.

use crate::adapters::Capabilities;
use crate::config::SiteStatus;
use crate::error::AdapterError;
use crate::ssh::RemoteCommand;

/// systemd can do everything and reports reliable status.
pub fn capabilities() -> Capabilities {
    Capabilities {
        can_start_stop: true,
        can_restart: true,
        has_logs: true,
        status_is_reliable: true,
    }
}

/// `systemctl show <unit> --property=...`. `show` is chosen over `status` because
/// its `key=value` output is stable and machine-parseable.
pub fn status_command(unit: &str) -> RemoteCommand {
    RemoteCommand::new(
        "systemctl show {} --property=ActiveState,SubState,ActiveEnterTimestamp",
        vec![unit.to_string()],
    )
}

/// `sudo systemctl start <unit>`.
pub fn start_command(unit: &str) -> RemoteCommand {
    RemoteCommand::new("sudo systemctl start {}", vec![unit.to_string()])
}

/// `sudo systemctl stop <unit>`.
pub fn stop_command(unit: &str) -> RemoteCommand {
    RemoteCommand::new("sudo systemctl stop {}", vec![unit.to_string()])
}

/// `sudo systemctl restart <unit>`.
pub fn restart_command(unit: &str) -> RemoteCommand {
    RemoteCommand::new("sudo systemctl restart {}", vec![unit.to_string()])
}

/// `journalctl -u <unit> -f -n 200`.
pub fn logs_command(unit: &str) -> RemoteCommand {
    RemoteCommand::new("journalctl -u {} -f -n 200", vec![unit.to_string()])
}

/// Parse `systemctl show` `key=value` lines into a [`SiteStatus`].
pub fn parse_status(output: &str) -> Result<SiteStatus, AdapterError> {
    let mut active_state = None;
    let mut sub_state = None;
    let mut enter_ts = None;

    for line in output.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("ActiveState=") {
            active_state = Some(v.to_string());
        } else if let Some(v) = line.strip_prefix("SubState=") {
            sub_state = Some(v.to_string());
        } else if let Some(v) = line.strip_prefix("ActiveEnterTimestamp=") {
            enter_ts = Some(v.to_string());
        }
    }

    let active_state = active_state.ok_or_else(|| AdapterError::Unparseable {
        tool: "systemctl show".into(),
        detail: "no ActiveState in output".into(),
    })?;

    let since = enter_ts
        .filter(|s| !s.is_empty())
        .and_then(|s| parse_systemd_timestamp(&s));

    Ok(match active_state.as_str() {
        "active" => SiteStatus::Running { since },
        "inactive" => SiteStatus::Stopped,
        "failed" => SiteStatus::Failed {
            reason: format!(
                "unit is in failed state ({})",
                sub_state.unwrap_or_else(|| "unknown sub-state".into())
            ),
        },
        "activating" | "deactivating" | "reloading" => SiteStatus::Checking,
        other => SiteStatus::Unknown {
            reason: format!("systemd reported ActiveState={other}"),
        },
    })
}

/// Parse systemd's timestamp form, e.g. `Sat 2026-07-11 14:03:22 UTC`. Returns
/// `None` on anything unexpected rather than failing the whole status parse.
fn parse_systemd_timestamp(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    // Systemd prints "Www YYYY-MM-DD HH:MM:SS TZ". Strip the leading weekday.
    let without_dow = s.split_once(' ')?.1;
    // Try common zones; systemd usually prints the machine's zone name.
    for fmt in ["%Y-%m-%d %H:%M:%S %Z", "%Y-%m-%d %H:%M:%S UTC"] {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(without_dow, fmt) {
            return Some(chrono::DateTime::from_naive_utc_and_offset(
                naive,
                chrono::Utc,
            ));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACTIVE: &str =
        "ActiveState=active\nSubState=running\nActiveEnterTimestamp=Sat 2026-07-11 14:03:22 UTC";
    const INACTIVE: &str = "ActiveState=inactive\nSubState=dead\nActiveEnterTimestamp=";
    const FAILED: &str = "ActiveState=failed\nSubState=failed\nActiveEnterTimestamp=";

    #[test]
    fn active_is_running_with_since() {
        match parse_status(ACTIVE).unwrap() {
            SiteStatus::Running { since } => assert!(since.is_some()),
            other => panic!("expected Running, got {other:?}"),
        }
    }

    #[test]
    fn inactive_is_stopped() {
        assert_eq!(parse_status(INACTIVE).unwrap(), SiteStatus::Stopped);
    }

    #[test]
    fn failed_is_failed_with_substate() {
        match parse_status(FAILED).unwrap() {
            SiteStatus::Failed { reason } => assert!(reason.contains("failed")),
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn missing_active_state_is_a_parse_error() {
        assert!(matches!(
            parse_status("SubState=running").unwrap_err(),
            AdapterError::Unparseable { .. }
        ));
    }

    #[test]
    fn start_command_escapes_unit() {
        let c = start_command("web; rm -rf /");
        assert_eq!(c.render(), "sudo systemctl start 'web; rm -rf /'");
    }
}
