use popush_core::command_log::CommandLogEntry;
use popush_core::config::{GitStatus, ServerConfig, SiteConfig, SiteStatus};
use popush_core::error::AppError;
use popush_core::ids::{PipelineId, ServerId, SiteId};
use popush_core::wizard::{Check, CheckStatus, Fix};
use tauri::State;

use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct ConnectionResult {
    pub ok: bool,
    pub latency_ms: Option<u64>,
}

#[tauri::command]
pub async fn list_servers(state: State<'_, AppState>) -> Result<Vec<ServerConfig>, AppError> {
    Ok(state.servers())
}

#[tauri::command]
pub async fn list_sites(
    state: State<'_, AppState>,
    server_id: ServerId,
) -> Result<Vec<SiteConfig>, AppError> {
    Ok(state
        .servers()
        .into_iter()
        .find(|s| s.id == server_id)
        .map(|s| s.sites)
        .unwrap_or_default())
}

#[tauri::command]
pub async fn test_connection(
    state: State<'_, AppState>,
    server_id: ServerId,
) -> Result<ConnectionResult, AppError> {
    match crate::ops::test_connection(&state, &server_id).await {
        Ok(latency_ms) => Ok(ConnectionResult {
            ok: true,
            latency_ms: Some(latency_ms),
        }),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn get_site_status(
    state: State<'_, AppState>,
    site_id: SiteId,
) -> Result<SiteStatus, AppError> {
    Ok(crate::ops::site_status(&state, &site_id).await)
}

#[tauri::command]
pub async fn site_action(
    state: State<'_, AppState>,
    site_id: SiteId,
    action: String,
) -> Result<(), String> {
    crate::ops::site_action(&state, &site_id, &action)
        .await
        .map_err(friendly)
}

#[tauri::command]
pub async fn get_site_logs(state: State<'_, AppState>, site_id: SiteId) -> Result<String, String> {
    crate::ops::site_logs(&state, &site_id)
        .await
        .map_err(friendly)
}

/// Keep a server's SSH password for this session only. An empty password
/// forgets it. Nothing is ever written to disk.
#[tauri::command]
pub async fn set_ssh_password(
    state: State<'_, AppState>,
    server_id: ServerId,
    password: String,
    save: bool,
) -> Result<(), AppError> {
    state.set_ssh_password(server_id, password, save);
    Ok(())
}

#[tauri::command]
pub async fn ssh_password_saved(
    state: State<'_, AppState>,
    server_id: ServerId,
) -> Result<bool, AppError> {
    Ok(state.ssh_password_is_saved(&server_id))
}

/// Render an error the way a person would say it: what happened, then the
/// one thing to do about it.
fn friendly(e: AppError) -> String {
    let m = e.user_message();
    let mut out = m.headline;
    match m.next_action {
        popush_core::error::NextAction::RunCommand { command } => {
            out.push_str(&format!(" Fix: run `{command}` in a terminal."));
        }
        popush_core::error::NextAction::Advice { text } => {
            out.push(' ');
            out.push_str(&text);
        }
        _ => {}
    }
    out
}

#[tauri::command]
pub async fn git_status(
    state: State<'_, AppState>,
    site_id: SiteId,
) -> Result<GitStatus, AppError> {
    let site = find_site(&state, &site_id).ok_or_else(|| {
        AppError::Config(popush_core::error::ConfigError::InvalidField {
            field: "site_id".into(),
            problem: format!("no site with id `{}`", site_id.0),
        })
    })?;
    let local = site.local_path.ok_or_else(|| {
        AppError::Git(popush_core::error::GitError::Operation {
            detail: "this site has no local_path configured".into(),
        })
    })?;
    crate::git::status(&local, &site.git_remote).map_err(AppError::Git)
}

#[tauri::command]
pub async fn start_deploy(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    server_id: ServerId,
    site_id: SiteId,
    commit_message: Option<String>,
) -> Result<(), String> {
    use tauri::Manager;
    // Connect up front so a bad connection is reported immediately rather than
    // deep inside the pipeline.
    let (pool, site, service) = crate::ops::connect_site(&state, &site_id)
        .await
        .map_err(friendly)?;
    let local_path = site.local_path.clone().unwrap_or_default();
    let message = commit_message.unwrap_or_default();
    let pipeline_id = PipelineId::new();

    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let ctx = crate::pipeline::ship::ShipContext {
            app: app.clone(),
            state: &state,
            pool: &pool,
            server_id,
            site,
            service,
            local_path,
            files: Vec::new(),
            message,
            pipeline_id,
        };
        crate::pipeline::run_pipeline(ctx).await;
    });
    Ok(())
}

#[tauri::command]
pub async fn cancel_pipeline(
    state: State<'_, AppState>,
    pipeline_id: PipelineId,
) -> Result<(), AppError> {
    state.cancel(&pipeline_id);
    Ok(())
}

#[tauri::command]
pub async fn run_wizard_check(
    state: State<'_, AppState>,
    check: Check,
    site_id: Option<SiteId>,
) -> Result<CheckStatus, AppError> {
    let ssh_dir = ssh_dir();
    let repo = site_id
        .and_then(|id| find_site(&state, &id))
        .and_then(|s| s.local_path);
    Ok(crate::wizard::run_local_check(
        check,
        &ssh_dir,
        repo.as_deref(),
    ))
}

#[tauri::command]
pub async fn apply_wizard_fix(
    state: State<'_, AppState>,
    fix: Fix,
    site_id: Option<SiteId>,
) -> Result<(), AppError> {
    let repo = site_id
        .and_then(|id| find_site(&state, &id))
        .and_then(|s| s.local_path);
    crate::wizard::apply_fix(&fix, repo.as_deref())
        .map_err(|detail| AppError::Git(popush_core::error::GitError::Operation { detail }))
}

#[tauri::command]
pub async fn add_server(state: State<'_, AppState>, server: ServerConfig) -> Result<(), AppError> {
    state.add_or_update_server(server).map_err(AppError::Config)
}

#[tauri::command]
pub async fn add_site(
    state: State<'_, AppState>,
    server_id: ServerId,
    site: SiteConfig,
) -> Result<(), AppError> {
    state.add_site(&server_id, site).map_err(AppError::Config)
}

#[tauri::command]
pub async fn config_error(state: State<'_, AppState>) -> Result<Option<String>, AppError> {
    Ok(state.config_error())
}

#[tauri::command]
pub async fn import_config(state: State<'_, AppState>, toml: String) -> Result<usize, String> {
    // Return a plain readable message so the UI can show exactly what is wrong
    // (a TOML syntax error names its line; a validation error names its field).
    state.import_config(&toml).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_site(state: State<'_, AppState>, site_id: SiteId) -> Result<(), AppError> {
    state.remove_site(&site_id).map_err(AppError::Config)?;
    Ok(())
}

#[tauri::command]
pub async fn remove_server(
    state: State<'_, AppState>,
    server_id: ServerId,
) -> Result<(), AppError> {
    state.remove_server(&server_id).map_err(AppError::Config)?;
    Ok(())
}

#[tauri::command]
pub async fn get_config(
    state: State<'_, AppState>,
) -> Result<popush_core::config::Config, AppError> {
    Ok(state.config_snapshot())
}

#[tauri::command]
pub async fn config_file_path() -> Result<String, AppError> {
    let path = crate::state::ensure_config_file().map_err(AppError::Config)?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn command_log(state: State<'_, AppState>) -> Result<Vec<CommandLogEntry>, AppError> {
    Ok(state.command_log())
}

#[tauri::command]
pub async fn git_commit_and_push(
    state: State<'_, AppState>,
    site_id: SiteId,
    message: String,
    files: Vec<std::path::PathBuf>,
) -> Result<String, AppError> {
    let site = find_site(&state, &site_id).ok_or_else(|| {
        AppError::Config(popush_core::error::ConfigError::InvalidField {
            field: "site_id".into(),
            problem: format!("no site with id `{}`", site_id.0),
        })
    })?;
    let local = site.local_path.clone().ok_or_else(|| {
        AppError::Git(popush_core::error::GitError::Operation {
            detail: "this site has no local_path configured".into(),
        })
    })?;
    let sha = crate::git::stage_and_commit(&local, &message, &files).map_err(AppError::Git)?;
    crate::git::push(&local, &site.git_remote, &site.git_branch).map_err(AppError::Git)?;
    Ok(sha)
}

#[tauri::command]
pub async fn set_github_token(token: String) -> Result<(), AppError> {
    crate::github::store_token(&token).map_err(|e| {
        AppError::Config(popush_core::error::ConfigError::Unreadable {
            path: "system keyring".into(),
            detail: e.to_string(),
        })
    })
}

#[tauri::command]
pub async fn clear_github_token() -> Result<(), AppError> {
    crate::github::clear_token().map_err(|e| {
        AppError::Config(popush_core::error::ConfigError::Unreadable {
            path: "system keyring".into(),
            detail: e.to_string(),
        })
    })
}

#[tauri::command]
pub async fn github_repo_info(
    owner: String,
    repo: String,
    branch: String,
) -> Result<GithubInfo, AppError> {
    let Some(client) = crate::github::GitHubClient::from_keyring() else {
        return Ok(GithubInfo::default());
    };
    Ok(GithubInfo {
        latest_commit: client.latest_commit(&owner, &repo, &branch).await,
        ci_status: client.ci_status(&owner, &repo, &branch).await,
        open_pr_count: client.open_pr_count(&owner, &repo).await,
        token_present: true,
    })
}

#[derive(serde::Serialize, Default)]
pub struct GithubInfo {
    pub latest_commit: Option<popush_core::github::LatestCommit>,
    #[serde(default)]
    pub ci_status: popush_core::github::CiStatus,
    pub open_pr_count: usize,
    pub token_present: bool,
}

#[tauri::command]
pub async fn app_credit() -> Result<Credit, AppError> {
    Ok(Credit {
        author: popush_core::AUTHOR.to_string(),
        version: popush_core::VERSION.to_string(),
    })
}

#[derive(serde::Serialize)]
pub struct Credit {
    pub author: String,
    pub version: String,
}

fn ssh_dir() -> std::path::PathBuf {
    directories::UserDirs::new()
        .map(|d| d.home_dir().join(".ssh"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.ssh"))
}

fn find_site(state: &AppState, site_id: &SiteId) -> Option<SiteConfig> {
    state
        .servers()
        .into_iter()
        .flat_map(|s| s.sites)
        .find(|s| &s.id == site_id)
}
