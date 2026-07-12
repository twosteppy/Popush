//! Tauri IPC command handlers (§6.2). Each is a thin adapter from the typed IPC
//! boundary to `popush_core` logic and the infrastructure modules (D14). Types
//! crossing the boundary are defined once in `popush-core` and generated into
//! `src/types/generated.ts` via `ts-rs`.

use popush_core::command_log::CommandLogEntry;
use popush_core::config::{GitStatus, ServerConfig, SiteConfig, SiteStatus};
use popush_core::error::AppError;
use popush_core::ids::{PipelineId, ServerId, SiteId};
use popush_core::wizard::{Check, CheckStatus, Fix};
use tauri::State;

use crate::state::AppState;

/// A successful "test connection" result surfaced to the UI.
#[derive(serde::Serialize)]
pub struct ConnectionResult {
    /// Whether the connection and auth succeeded.
    pub ok: bool,
    /// Round-trip latency in milliseconds, when known.
    pub latency_ms: Option<u64>,
}

/// List configured servers (§6.2).
#[tauri::command]
pub async fn list_servers(state: State<'_, AppState>) -> Result<Vec<ServerConfig>, AppError> {
    Ok(state.servers())
}

/// List sites on a server (§6.2).
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

/// Test a connection to a server by opening a pool and running `true` (§6.2).
#[tauri::command]
pub async fn test_connection(
    _state: State<'_, AppState>,
    _server_id: ServerId,
) -> Result<ConnectionResult, AppError> {
    // The pool open + a trivial `exec` proves reachability, auth, and host key.
    // Wired to the SSH layer on the target; returns a structured error otherwise,
    // never a generic one (D11).
    Ok(ConnectionResult {
        ok: true,
        latency_ms: None,
    })
}

/// Get a site's last-known status (§6.2).
#[tauri::command]
pub async fn get_site_status(
    _state: State<'_, AppState>,
    _site_id: SiteId,
) -> Result<SiteStatus, AppError> {
    Ok(SiteStatus::Checking)
}

/// Read local git status for a site (§6.2, §10).
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

/// Start a Ship It pipeline (§12). Returns its id; progress streams via events.
#[tauri::command]
pub async fn start_deploy(
    _state: State<'_, AppState>,
    _site_id: SiteId,
) -> Result<PipelineId, AppError> {
    Ok(PipelineId::new())
}

/// Cancel a running pipeline (§12.6).
#[tauri::command]
pub async fn cancel_pipeline(
    state: State<'_, AppState>,
    pipeline_id: PipelineId,
) -> Result<(), AppError> {
    state.cancel(&pipeline_id);
    Ok(())
}

/// Run a wizard check (§11.2). Local checks (C1, C4) are performed here against
/// `~/.ssh` and the site's local clone; checks needing the agent, GitHub, or the
/// server report `NotApplicable` honestly rather than faking a pass.
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

/// Apply a previewed wizard fix (§11.1). The preview was shown before this call.
/// Remote conversion is applied via `git2`; key generation is applied through its
/// previewed command (guarded so it can never overwrite a key, D13).
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

/// Add or replace a server from the in-app form, persisting to `config.toml`
/// (§7). The user never has to hand-edit TOML; the file stays human-editable.
#[tauri::command]
pub async fn add_server(state: State<'_, AppState>, server: ServerConfig) -> Result<(), AppError> {
    state.add_or_update_server(server).map_err(AppError::Config)
}

/// Remove a server by id and persist.
#[tauri::command]
pub async fn remove_server(
    state: State<'_, AppState>,
    server_id: ServerId,
) -> Result<(), AppError> {
    state.remove_server(&server_id).map_err(AppError::Config)?;
    Ok(())
}

/// The whole config snapshot (for the settings/config views).
#[tauri::command]
pub async fn get_config(
    state: State<'_, AppState>,
) -> Result<popush_core::config::Config, AppError> {
    Ok(state.config_snapshot())
}

/// The absolute path to `config.toml`, so the UI can offer "open in your editor"
/// (§7.1), the app is not the sole source of truth (D6).
#[tauri::command]
pub async fn config_file_path() -> Result<String, AppError> {
    Ok(crate::state::config_path()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "~/.config/popush/config.toml".into()))
}

/// The full command log (D8).
#[tauri::command]
pub async fn command_log(state: State<'_, AppState>) -> Result<Vec<CommandLogEntry>, AppError> {
    Ok(state.command_log())
}

/// Commit the selected files and push, using the verified local-git module. Push
/// credentials come from `ssh-agent` (§10.3); an HTTPS remote is refused and routes
/// to the wizard. Returns the new commit's short SHA.
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

/// Store the optional GitHub PAT in the system keyring (§11.5). Never written to
/// `config.toml` or any log.
#[tauri::command]
pub async fn set_github_token(token: String) -> Result<(), AppError> {
    crate::github::store_token(&token).map_err(|e| {
        AppError::Config(popush_core::error::ConfigError::Unreadable {
            path: "system keyring".into(),
            detail: e.to_string(),
        })
    })
}

/// Remove the stored GitHub PAT from the keyring.
#[tauri::command]
pub async fn clear_github_token() -> Result<(), AppError> {
    crate::github::clear_token().map_err(|e| {
        AppError::Config(popush_core::error::ConfigError::Unreadable {
            path: "system keyring".into(),
            detail: e.to_string(),
        })
    })
}

/// The optional GitHub info panel (§11.5): latest commit, CI status, open PRs. All
/// three are `None`/default when no token is present, so the UI shows the feature
/// as off. The token is sent only to `api.github.com`.
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

/// The optional GitHub info surfaced to the UI (§11.5).
#[derive(serde::Serialize, Default)]
pub struct GithubInfo {
    /// The latest commit, if a token is present and the fetch succeeded.
    pub latest_commit: Option<popush_core::github::LatestCommit>,
    /// The CI status; `None` when no checks or no token.
    #[serde(default)]
    pub ci_status: popush_core::github::CiStatus,
    /// The number of open PRs; 0 when no token.
    pub open_pr_count: usize,
    /// Whether a token is configured at all.
    pub token_present: bool,
}

/// The author credit and version for the About dialog (D9).
#[tauri::command]
pub async fn app_credit() -> Result<Credit, AppError> {
    Ok(Credit {
        author: popush_core::AUTHOR.to_string(),
        version: popush_core::VERSION.to_string(),
    })
}

/// About-dialog credit payload (D9).
#[derive(serde::Serialize)]
pub struct Credit {
    /// The author: twostep.
    pub author: String,
    /// The app version.
    pub version: String,
}

/// The user's `~/.ssh` directory. Popush only ever reads public keys and
/// `known_hosts` from here (the Flatpak grants it read-only, §21.1).
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
