//! Optional GitHub API client (Phase 10), the I/O half.
//!
//! Strictly optional: every core feature works with no token. When the user
//! supplies a fine-grained read-only PAT, it is stored in the **system keyring
//! only** (never `config.toml`, never a log), and requests go to `api.github.com`
//! and nowhere else. The JSON parsing is `popush_core::github`; this layer holds
//! the keyring access and the HTTP calls.

use popush_core::github::{
    parse_ci_status, parse_latest_commit, parse_open_pr_count, CiStatus, LatestCommit,
};

/// The keyring service and account under which the token is stored. Using a fixed
/// pair means the token lives at exactly one keyring location and nowhere else.
const KEYRING_SERVICE: &str = "dev.popush.Popush";
const KEYRING_ACCOUNT: &str = "github-pat";

/// The single host Popush ever sends the token to.
const API_BASE: &str = "https://api.github.com";

/// Store the PAT in the system keyring. This is the only place Popush ever
/// writes it; it never touches `config.toml`, history, or logs.
pub fn store_token(token: &str) -> Result<(), keyring::Error> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)?.set_password(token)
}

/// Read the PAT from the keyring, or `None` if the user has not set one.
pub fn get_token() -> Option<String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .ok()?
        .get_password()
        .ok()
}

/// Remove the PAT from the keyring.
pub fn clear_token() -> Result<(), keyring::Error> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)?.delete_credential()
}

/// A minimal GitHub API client bound to a token. Constructed only when a token is
/// present, so the type system reflects "no token, no client".
pub struct GitHubClient {
    http: reqwest::Client,
    token: String,
}

impl GitHubClient {
    /// Build a client if a token is stored, else `None`. The caller shows the
    /// optional features only when this is `Some`.
    pub fn from_keyring() -> Option<Self> {
        let token = get_token()?;
        Some(Self {
            http: reqwest::Client::new(),
            token,
        })
    }

    async fn get(&self, path: &str) -> Result<String, reqwest::Error> {
        // The token is sent to api.github.com and nowhere else. The path
        // is always a fixed template built here, never a caller-supplied URL.
        self.http
            .get(format!("{API_BASE}{path}"))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "Popush")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
    }

    /// The latest commit on `branch` of `owner/repo`.
    pub async fn latest_commit(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> Option<LatestCommit> {
        let body = self
            .get(&format!("/repos/{owner}/{repo}/commits/{branch}"))
            .await
            .ok()?;
        parse_latest_commit(&body)
    }

    /// The CI status for `git_ref` on `owner/repo`.
    pub async fn ci_status(&self, owner: &str, repo: &str, git_ref: &str) -> CiStatus {
        match self
            .get(&format!(
                "/repos/{owner}/{repo}/commits/{git_ref}/check-runs"
            ))
            .await
        {
            Ok(body) => parse_ci_status(&body),
            Err(_) => CiStatus::None,
        }
    }

    /// The number of open pull requests on `owner/repo`.
    pub async fn open_pr_count(&self, owner: &str, repo: &str) -> usize {
        match self
            .get(&format!(
                "/repos/{owner}/{repo}/pulls?state=open&per_page=100"
            ))
            .await
        {
            Ok(body) => parse_open_pr_count(&body),
            Err(_) => 0,
        }
    }
}
