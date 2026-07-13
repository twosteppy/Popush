use std::io::Write;
use std::path::PathBuf;

use popush_core::config::{ServerConfig, ServiceConfig, SiteConfig, SiteStatus};
use popush_core::error::{AppError, ConfigError};
use popush_core::ids::{ServerId, SiteId};
use popush_core::ssh::{parse_known_hosts, KnownHost, RemoteCommand};

use crate::ssh::SshPool;
use crate::state::AppState;

fn ssh_dir() -> PathBuf {
    directories::UserDirs::new()
        .map(|d| d.home_dir().join(".ssh"))
        .unwrap_or_else(|| PathBuf::from("~/.ssh"))
}

fn known_hosts_path() -> PathBuf {
    ssh_dir().join("known_hosts")
}

fn read_known_hosts() -> Vec<KnownHost> {
    std::fs::read_to_string(known_hosts_path())
        .map(|s| parse_known_hosts(&s))
        .unwrap_or_default()
}

/// Record a trust-on-first-use host key in ~/.ssh/known_hosts.
fn append_known_host(kh: &KnownHost) {
    let dir = ssh_dir();
    let _ = std::fs::create_dir_all(&dir);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
    }
    let path = known_hosts_path();
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(f, "{} {} {}", kh.host, kh.key_type, kh.key_base64);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
}

fn no_site(site_id: &SiteId) -> AppError {
    AppError::Config(ConfigError::InvalidField {
        field: "site_id".into(),
        problem: format!("no site with id `{}`", site_id.0),
    })
}

fn find_server_and_site(state: &AppState, site_id: &SiteId) -> Option<(ServerConfig, SiteConfig)> {
    for server in state.servers() {
        if let Some(site) = server.sites.iter().find(|s| &s.id == site_id) {
            return Some((server.clone(), site.clone()));
        }
    }
    None
}

async fn connect(server: ServerConfig) -> Result<SshPool, AppError> {
    let known = read_known_hosts();
    let (pool, new_host) = SshPool::connect_tofu(server, known)
        .await
        .map_err(AppError::Ssh)?;
    if let Some(kh) = new_host {
        append_known_host(&kh);
    }
    Ok(pool)
}

/// Connect to a server by id and run a trivial command to prove it works.
pub async fn test_connection(state: &AppState, server_id: &ServerId) -> Result<u64, AppError> {
    let server = state
        .servers()
        .into_iter()
        .find(|s| &s.id == server_id)
        .ok_or_else(|| {
            AppError::Config(ConfigError::InvalidField {
                field: "server_id".into(),
                problem: format!("no server with id `{}`", server_id.0),
            })
        })?;
    let start = std::time::Instant::now();
    let pool = connect(server).await?;
    pool.exec(RemoteCommand::literal("echo popush-ok"))
        .await
        .map_err(AppError::Ssh)?;
    Ok(start.elapsed().as_millis() as u64)
}

/// Connect for a site, returning the pool and its resolved service.
pub async fn connect_site(
    state: &AppState,
    site_id: &SiteId,
) -> Result<(SshPool, SiteConfig, ServiceConfig), AppError> {
    let (server, site) = find_server_and_site(state, site_id).ok_or_else(|| no_site(site_id))?;
    let service = site.resolve_service().map_err(|(field, problem)| {
        AppError::Config(ConfigError::InvalidField {
            field: field.to_string(),
            problem,
        })
    })?;
    let pool = connect(server).await?;
    Ok((pool, site, service))
}

/// Live status for a site, reduced to online or offline.
///
/// The public URL is the source of truth: if the site answers over HTTP it is
/// online, otherwise it is offline. A broken certificate counts as offline,
/// because that is what a visitor sees. Sites without a URL fall back to
/// asking the service manager over SSH.
pub async fn site_status(state: &AppState, site_id: &SiteId) -> SiteStatus {
    let Some((_, site)) = find_server_and_site(state, site_id) else {
        return SiteStatus::Stopped;
    };
    if let Some(url) = site.live_url.as_deref() {
        if url.starts_with("https://") || url.starts_with("http://") {
            return http_status(url).await;
        }
    }
    let Ok((pool, site, service)) = connect_site(state, site_id).await else {
        return SiteStatus::Stopped;
    };
    match crate::adapters::status(&pool, &service, &site.remote_path.to_string_lossy()).await {
        Ok(SiteStatus::Running { since }) => SiteStatus::Running { since },
        _ => SiteStatus::Stopped,
    }
}

async fn http_status(url: &str) -> SiteStatus {
    let Ok(client) = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(6))
        .timeout(std::time::Duration::from_secs(10))
        .user_agent(concat!("Popush/", env!("CARGO_PKG_VERSION")))
        .build()
    else {
        return SiteStatus::Stopped;
    };
    match client.get(url).send().await {
        // Any answer below 500 means the site is up and serving; a 5xx means
        // it is reachable but broken, which a visitor experiences as down.
        Ok(resp) if resp.status().as_u16() < 500 => SiteStatus::Running { since: None },
        _ => SiteStatus::Stopped,
    }
}

/// Run start/stop/restart for a site over SSH.
pub async fn site_action(state: &AppState, site_id: &SiteId, action: &str) -> Result<(), AppError> {
    let (pool, site, service) = connect_site(state, site_id).await?;
    crate::adapters::run_action(&pool, &service, &site.remote_path.to_string_lossy(), action)
        .await
        .map_err(AppError::Adapter)
}
