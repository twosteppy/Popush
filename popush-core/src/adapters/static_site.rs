//! Static-site adapter (§9.5) — the honest one.
//!
//! A static site served by nginx has no process of its own. Its "status" is
//! whether files exist and, if a `health_check_url` is configured, whether an
//! HTTP `HEAD` succeeds. Without a health check, status is **not reliable**
//! (`status_is_reliable = false`) and the UI shows **amber Unknown** (D12). A
//! green light that means "the folder exists" is worse than an honest amber one,
//! so this adapter refuses to earn green from a directory listing alone.

use crate::adapters::Capabilities;
use crate::config::SiteStatus;
use crate::ssh::RemoteCommand;

/// Static sites cannot be started, stopped, or restarted; status is reliable only
/// with a health check (§9.5). The `has_health_check` argument decides that.
pub fn capabilities(has_health_check: bool) -> Capabilities {
    Capabilities {
        can_start_stop: false,
        can_restart: false,
        // Logs only if an nginx log path is configured; conservatively false here.
        has_logs: false,
        status_is_reliable: has_health_check,
    }
}

/// `test -d <root> && ls -1 <root> | head -1` (§9.5). Confirms the web root exists
/// and is non-empty. This alone never earns green.
pub fn presence_command(web_root: &str) -> RemoteCommand {
    RemoteCommand::new(
        "test -d {} && ls -1 {} | head -1",
        vec![web_root.to_string(), web_root.to_string()],
    )
}

/// The outcome of the file-presence check, before any HTTP verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresenceOutcome {
    /// The web root exists and has at least one entry.
    Present,
    /// The web root is missing or empty.
    Absent,
}

/// Interpret the presence command's exit code and stdout.
pub fn interpret_presence(exit_code: i32, stdout: &str) -> PresenceOutcome {
    if exit_code == 0 && !stdout.trim().is_empty() {
        PresenceOutcome::Present
    } else {
        PresenceOutcome::Absent
    }
}

/// Combine file presence with an optional HTTP health verdict into a status.
///
/// The honesty rule (D12): files present but no health check → **amber Unknown**,
/// never green. Only a passing health check earns `Running`.
pub fn resolve_status(presence: PresenceOutcome, health: Option<HealthVerdict>) -> SiteStatus {
    match (presence, health) {
        (PresenceOutcome::Absent, _) => SiteStatus::Failed {
            reason: "the web root is missing or empty on the server".into(),
        },
        (PresenceOutcome::Present, Some(HealthVerdict::Ok)) => SiteStatus::Running { since: None },
        (PresenceOutcome::Present, Some(HealthVerdict::Http { code })) => SiteStatus::Failed {
            reason: format!("the health check returned HTTP {code}"),
        },
        (PresenceOutcome::Present, Some(HealthVerdict::Unreachable)) => SiteStatus::Unknown {
            reason: "the files are on disk but the health check URL could not be reached".into(),
        },
        (PresenceOutcome::Present, None) => SiteStatus::Unknown {
            reason: "the files exist, but no health check is configured, so Popush cannot confirm the site is actually being served".into(),
        },
    }
}

impl HealthVerdict {
    /// Classify an HTTP status code from a `HEAD` to the health check URL. A 2xx
    /// earns `Ok`; anything else is reported honestly with its code (§9.5). The
    /// binary calls this after performing the request.
    pub fn from_http_status(code: u16) -> Self {
        if (200..300).contains(&code) {
            HealthVerdict::Ok
        } else {
            HealthVerdict::Http { code }
        }
    }
}

/// The verdict of an HTTP `HEAD` to the configured `health_check_url`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthVerdict {
    /// A 2xx response.
    Ok,
    /// A non-2xx HTTP response.
    Http {
        /// The HTTP status code returned.
        code: u16,
    },
    /// No response at all.
    Unreachable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn present_without_health_check_is_amber_unknown_not_green() {
        // The single most important honesty test for this adapter (D12).
        let status = resolve_status(PresenceOutcome::Present, None);
        assert!(matches!(status, SiteStatus::Unknown { .. }));
    }

    #[test]
    fn present_with_passing_health_check_is_running() {
        let status = resolve_status(PresenceOutcome::Present, Some(HealthVerdict::Ok));
        assert!(matches!(status, SiteStatus::Running { .. }));
    }

    #[test]
    fn present_with_500_is_failed() {
        let status = resolve_status(
            PresenceOutcome::Present,
            Some(HealthVerdict::Http { code: 500 }),
        );
        match status {
            SiteStatus::Failed { reason } => assert!(reason.contains("500")),
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn missing_root_is_failed() {
        let status = resolve_status(PresenceOutcome::Absent, Some(HealthVerdict::Ok));
        assert!(matches!(status, SiteStatus::Failed { .. }));
    }

    #[test]
    fn capabilities_reflect_health_check_presence() {
        assert!(!capabilities(false).status_is_reliable);
        assert!(capabilities(true).status_is_reliable);
        assert!(!capabilities(true).can_restart);
    }

    #[test]
    fn http_status_classification() {
        assert_eq!(HealthVerdict::from_http_status(200), HealthVerdict::Ok);
        assert_eq!(HealthVerdict::from_http_status(204), HealthVerdict::Ok);
        assert_eq!(
            HealthVerdict::from_http_status(500),
            HealthVerdict::Http { code: 500 }
        );
        assert_eq!(
            HealthVerdict::from_http_status(404),
            HealthVerdict::Http { code: 404 }
        );
    }

    #[test]
    fn presence_command_escapes_root() {
        let c = presence_command("/var/www; rm -rf /");
        assert!(c.render().starts_with("test -d '/var/www; rm -rf /'"));
    }
}
