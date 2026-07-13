use popush_core::adapters::{docker, pm2, static_site, systemd, Capabilities};
use popush_core::config::{ServiceConfig, SiteStatus};
use popush_core::error::AdapterError;

use crate::ssh::SshPool;

pub fn capabilities(service: &ServiceConfig, has_health_check: bool) -> Capabilities {
    match service {
        ServiceConfig::Docker { .. } => docker::capabilities(),
        ServiceConfig::Systemd { .. } => systemd::capabilities(),
        ServiceConfig::Pm2 { .. } => pm2::capabilities(),
        ServiceConfig::Static { .. } => static_site::capabilities(has_health_check),
    }
}

/// Run a start/stop/restart action for a service and fail on a non-zero exit.
pub async fn run_action(
    pool: &SshPool,
    service: &ServiceConfig,
    remote_path: &str,
    action: &str,
) -> Result<(), AdapterError> {
    let cmd = match (service, action) {
        (ServiceConfig::Docker { .. }, "start") => docker::start_command(remote_path),
        (ServiceConfig::Docker { .. }, "stop") => docker::stop_command(remote_path),
        (ServiceConfig::Docker { .. }, "restart") => docker::restart_command(remote_path),
        (ServiceConfig::Systemd { unit }, "start") => systemd::start_command(unit),
        (ServiceConfig::Systemd { unit }, "stop") => systemd::stop_command(unit),
        (ServiceConfig::Systemd { unit }, "restart") => systemd::restart_command(unit),
        (ServiceConfig::Pm2 { app_name }, "start") => pm2::start_command(app_name),
        (ServiceConfig::Pm2 { app_name }, "stop") => pm2::stop_command(app_name),
        (ServiceConfig::Pm2 { app_name }, "restart") => pm2::restart_command(app_name),
        (ServiceConfig::Static { .. }, _) => {
            return Err(AdapterError::Unsupported {
                operation: action.to_string(),
                service_type: "static".into(),
            })
        }
        (_, other) => {
            return Err(AdapterError::Unsupported {
                operation: other.to_string(),
                service_type: "service".into(),
            })
        }
    };
    let out = pool.exec(cmd).await.map_err(AdapterError::Ssh)?;
    if out.exit_code != 0 {
        return Err(AdapterError::Unparseable {
            tool: action.to_string(),
            detail: if out.stderr.trim().is_empty() {
                format!("command exited {}", out.exit_code)
            } else {
                out.stderr
            },
        });
    }
    Ok(())
}

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
            Ok(static_site::resolve_status(presence, None))
        }
    }
}
