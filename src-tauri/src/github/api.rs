use popush_core::github::{
    parse_ci_status, parse_latest_commit, parse_open_pr_count, CiStatus, LatestCommit,
};

const KEYRING_SERVICE: &str = "dev.popush.Popush";
const KEYRING_ACCOUNT: &str = "github-pat";

const API_BASE: &str = "https://api.github.com";

fn enc(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for b in segment.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

pub fn store_token(token: &str) -> Result<(), keyring::Error> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)?.set_password(token)
}

pub fn get_token() -> Option<String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .ok()?
        .get_password()
        .ok()
}

pub fn clear_token() -> Result<(), keyring::Error> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)?.delete_credential()
}

pub struct GitHubClient {
    http: reqwest::Client,
    token: String,
}

impl GitHubClient {
    pub fn from_keyring() -> Option<Self> {
        let token = get_token()?;
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .ok()?;
        Some(Self { http, token })
    }

    async fn get(&self, path: &str) -> Result<String, reqwest::Error> {
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

    pub async fn latest_commit(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> Option<LatestCommit> {
        let body = self
            .get(&format!(
                "/repos/{}/{}/commits/{}",
                enc(owner),
                enc(repo),
                enc(branch)
            ))
            .await
            .ok()?;
        parse_latest_commit(&body)
    }

    pub async fn ci_status(&self, owner: &str, repo: &str, git_ref: &str) -> CiStatus {
        match self
            .get(&format!(
                "/repos/{}/{}/commits/{}/check-runs",
                enc(owner),
                enc(repo),
                enc(git_ref)
            ))
            .await
        {
            Ok(body) => parse_ci_status(&body),
            Err(_) => CiStatus::None,
        }
    }

    pub async fn open_pr_count(&self, owner: &str, repo: &str) -> usize {
        match self
            .get(&format!(
                "/repos/{}/{}/pulls?state=open&per_page=100",
                enc(owner),
                enc(repo)
            ))
            .await
        {
            Ok(body) => parse_open_pr_count(&body),
            Err(_) => 0,
        }
    }
}
