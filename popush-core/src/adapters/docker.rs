//! Docker Compose adapter (§9.2). The recommended setup.

use serde::Deserialize;

use crate::adapters::Capabilities;
use crate::config::SiteStatus;
use crate::error::AdapterError;
use crate::ssh::RemoteCommand;

/// Capabilities of the Docker adapter: full control, reliable status.
pub fn capabilities() -> Capabilities {
    Capabilities {
        can_start_stop: true,
        can_restart: true,
        has_logs: true,
        status_is_reliable: true,
    }
}

/// `docker compose ps --format json` in the site directory (§9.2).
pub fn status_command(remote_path: &str) -> RemoteCommand {
    RemoteCommand::new(
        "cd {} && docker compose ps --format json",
        vec![remote_path.to_string()],
    )
}

/// `docker compose up -d`.
pub fn start_command(remote_path: &str) -> RemoteCommand {
    RemoteCommand::new(
        "cd {} && docker compose up -d",
        vec![remote_path.to_string()],
    )
}

/// `docker compose down`.
pub fn stop_command(remote_path: &str) -> RemoteCommand {
    RemoteCommand::new(
        "cd {} && docker compose down",
        vec![remote_path.to_string()],
    )
}

/// `docker compose restart`.
pub fn restart_command(remote_path: &str) -> RemoteCommand {
    RemoteCommand::new(
        "cd {} && docker compose restart",
        vec![remote_path.to_string()],
    )
}

/// `docker compose logs -f --tail=200`.
pub fn logs_command(remote_path: &str) -> RemoteCommand {
    RemoteCommand::new(
        "cd {} && docker compose logs -f --tail=200",
        vec![remote_path.to_string()],
    )
}

/// One container row from `docker compose ps --format json`.
///
/// Modern `docker compose` emits one JSON object per line (JSONL); older/other
/// versions emit a single JSON array. [`parse_status`] handles both.
#[derive(Debug, Deserialize)]
struct ContainerRow {
    #[serde(rename = "Name", default)]
    name: String,
    #[serde(rename = "State", default)]
    state: String,
    #[serde(rename = "Health", default)]
    health: String,
    #[serde(rename = "ExitCode", default)]
    exit_code: i64,
}

/// Parse `docker compose ps --format json` output into a [`SiteStatus`] using the
/// mapping in §9.2. An empty output means no containers → `Stopped`.
pub fn parse_status(output: &str) -> Result<SiteStatus, AdapterError> {
    let rows = parse_rows(output)?;
    if rows.is_empty() {
        return Ok(SiteStatus::Stopped);
    }

    let mut all_running_healthy = true;
    let mut all_exited_zero = true;
    let mut failure: Option<String> = None;

    for r in &rows {
        let running = r.state.eq_ignore_ascii_case("running");
        let healthy = r.health.is_empty()
            || r.health.eq_ignore_ascii_case("healthy")
            || r.health.eq_ignore_ascii_case("starting");
        if !(running && healthy) {
            all_running_healthy = false;
        }
        let exited_zero = r.state.eq_ignore_ascii_case("exited") && r.exit_code == 0;
        if !exited_zero {
            all_exited_zero = false;
        }
        // Any container exited non-zero, or unhealthy, is a described failure.
        if (r.state.eq_ignore_ascii_case("exited") && r.exit_code != 0)
            || r.health.eq_ignore_ascii_case("unhealthy")
        {
            failure.get_or_insert_with(|| {
                if r.health.eq_ignore_ascii_case("unhealthy") {
                    format!("container `{}` is unhealthy", r.name)
                } else {
                    format!("container `{}` exited with code {}", r.name, r.exit_code)
                }
            });
        }
    }

    if let Some(reason) = failure {
        return Ok(SiteStatus::Failed { reason });
    }
    if all_running_healthy {
        return Ok(SiteStatus::Running { since: None });
    }
    if all_exited_zero {
        return Ok(SiteStatus::Stopped);
    }
    // Mixed states with no explicit non-zero exit: name the container that is not
    // running, per §9.2 ("Mixed → Failed, describing which container is down").
    let down = rows
        .iter()
        .find(|r| !r.state.eq_ignore_ascii_case("running"))
        .map(|r| r.name.clone())
        .unwrap_or_else(|| "a container".into());
    Ok(SiteStatus::Failed {
        reason: format!("container `{down}` is not running"),
    })
}

fn parse_rows(output: &str) -> Result<Vec<ContainerRow>, AdapterError> {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    // Try a single JSON array first.
    if trimmed.starts_with('[') {
        return serde_json::from_str::<Vec<ContainerRow>>(trimmed).map_err(|e| {
            AdapterError::Unparseable {
                tool: "docker compose ps".into(),
                detail: e.to_string(),
            }
        });
    }
    // Otherwise JSONL: one object per non-empty line.
    let mut rows = Vec::new();
    for line in trimmed.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let row: ContainerRow =
            serde_json::from_str(line).map_err(|e| AdapterError::Unparseable {
                tool: "docker compose ps".into(),
                detail: format!("{e} in line: {line}"),
            })?;
        rows.push(row);
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real captured output shapes from `docker compose ps --format json`.
    // Modern compose emits JSONL (one object per line).
    const JSONL_HEALTHY: &str = r#"{"Name":"sterling-web-1","State":"running","Health":"healthy","ExitCode":0}
{"Name":"sterling-db-1","State":"running","Health":"healthy","ExitCode":0}"#;

    const JSONL_ONE_EXITED: &str = r#"{"Name":"sterling-web-1","State":"running","Health":"healthy","ExitCode":0}
{"Name":"sterling-worker-1","State":"exited","Health":"","ExitCode":1}"#;

    const JSONL_ALL_STOPPED: &str = r#"{"Name":"sterling-web-1","State":"exited","Health":"","ExitCode":0}
{"Name":"sterling-db-1","State":"exited","Health":"","ExitCode":0}"#;

    const JSONL_UNHEALTHY: &str =
        r#"{"Name":"sterling-web-1","State":"running","Health":"unhealthy","ExitCode":0}"#;

    // Older compose emitted a single JSON array.
    const ARRAY_HEALTHY: &str = r#"[{"Name":"web","State":"running","Health":"","ExitCode":0}]"#;

    #[test]
    fn all_running_healthy_is_running() {
        assert!(matches!(
            parse_status(JSONL_HEALTHY).unwrap(),
            SiteStatus::Running { .. }
        ));
    }

    #[test]
    fn one_exited_nonzero_is_failed_and_names_it() {
        match parse_status(JSONL_ONE_EXITED).unwrap() {
            SiteStatus::Failed { reason } => {
                assert!(reason.contains("sterling-worker-1"), "reason: {reason}");
                assert!(reason.contains("code 1"));
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn all_exited_zero_is_stopped() {
        assert_eq!(
            parse_status(JSONL_ALL_STOPPED).unwrap(),
            SiteStatus::Stopped
        );
    }

    #[test]
    fn unhealthy_is_failed() {
        match parse_status(JSONL_UNHEALTHY).unwrap() {
            SiteStatus::Failed { reason } => assert!(reason.contains("unhealthy")),
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn empty_output_is_stopped() {
        assert_eq!(parse_status("").unwrap(), SiteStatus::Stopped);
        assert_eq!(parse_status("\n\n").unwrap(), SiteStatus::Stopped);
    }

    #[test]
    fn legacy_json_array_is_supported() {
        assert!(matches!(
            parse_status(ARRAY_HEALTHY).unwrap(),
            SiteStatus::Running { .. }
        ));
    }

    #[test]
    fn garbage_output_is_a_named_parse_error() {
        let err = parse_status("{not json").unwrap_err();
        assert!(matches!(err, AdapterError::Unparseable { .. }));
    }

    #[test]
    fn commands_escape_the_path() {
        let c = status_command("/srv/a b; rm -rf /");
        // The dangerous path is a single quoted word; `rm` never runs.
        assert!(c.render().starts_with("cd '/srv/a b; rm -rf /'"));
    }
}
