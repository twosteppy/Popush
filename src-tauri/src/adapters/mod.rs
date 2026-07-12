//! Adapter runtime (§9): runs each adapter's command over SSH and hands the raw
//! output to the corresponding pure parser in `popush_core::adapters`. The parse —
//! where honest status is won (D12) — is tested in the core crate; this layer only
//! issues the command and records it in the log (D8).

use popush_core::adapters::{docker, pm2, static_site, systemd, Capabilities};
use popush_core::config::{ServiceConfig, SiteStatus};
use popush_core::error::AdapterError;

use crate::ssh::SshPool;

/// Capabilities for a service configuration (§9.1). Static reliability depends on
/// whether a health check is configured.
pub fn capabilities(service: &ServiceConfig, has_health_check: bool) -> Capabilities {
    match service {
        ServiceConfig::Docker { .. } => docker::capabilities(),
        ServiceConfig::Systemd { .. } => systemd::capabilities(),
        ServiceConfig::Pm2 { .. } => pm2::capabilities(),
        ServiceConfig::Static { .. } => static_site::capabilities(has_health_check),
    }
}

/// Check a site's status by running the adapter's status command and parsing it.
pub async fn status(
    pool: &SshPool,
    service: &ServiceConfig,
    remote_path: &str,
) -> Result<SiteStatus, AdapterError> {
    match service {
        ServiceConfig::Docker { .. } => {
            let out = pool
                .exec(docker::status_command(remote_path))
                .await
                .map_err(AdapterError::Ssh)?;
            docker::parse_status(&out.stdout)
        }
        ServiceConfig::Systemd { unit } => {
            let out = pool
                .exec(systemd::status_command(unit))
                .await
                .map_err(AdapterError::Ssh)?;
            systemd::parse_status(&out.stdout)
        }
        ServiceConfig::Pm2 { app_name } => {
            let out = pool
                .exec(pm2::status_command())
                .await
                .map_err(AdapterError::Ssh)?;
            pm2::parse_status(&out.stdout, app_name)
        }
        ServiceConfig::Static { web_root } => {
            let out = pool
                .exec(static_site::presence_command(&web_root.to_string_lossy()))
                .await
                .map_err(AdapterError::Ssh)?;
            let presence = static_site::interpret_presence(out.exit_code, &out.stdout);
            // The HTTP health verdict is fetched by the caller (it is not an SSH
            // command); here, with no health check, presence-only yields amber.
            Ok(static_site::resolve_status(presence, None))
        }
    }
}
