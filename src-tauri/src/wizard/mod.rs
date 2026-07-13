//! Setup-wizard I/O. The decisions, which fix, whether a key may be
//! generated, how to convert a remote, come from `popush_core::wizard`; this
//! layer performs the local filesystem and `git2` operations for the checks that
//! are local (C1 and C4) and applies previewed fixes. The remote checks (C3, C5,
//! C6, C7) run over SSH against a live server and are exercised by the integration
//! suite; this module implements the parts that are verifiable without a
//! server.

use std::path::{Path, PathBuf};

use popush_core::git::remote::classify_remote;
use popush_core::git::RemoteKind;
use popush_core::wizard::fixes::key_generation_fix;
use popush_core::wizard::{Check, CheckStatus, Fix};

/// The candidate local key paths C1 looks for, in preference order.
const KEY_CANDIDATES: [&str; 3] = ["id_ed25519", "id_ecdsa", "id_rsa"];

/// Run a local wizard check. Remote checks return a truthful `NotApplicable` here
/// and are performed by the SSH-backed path on the target.
pub fn run_local_check(check: Check, ssh_dir: &Path, repo_path: Option<&Path>) -> CheckStatus {
    match check {
        Check::LocalKeyExists => match find_local_key(ssh_dir) {
            Some(_) => CheckStatus::Pass,
            None => CheckStatus::Fail {
                what_is_wrong: "No SSH key was found in ~/.ssh.".into(),
            },
        },
        Check::LocalRemoteIsSsh => match repo_path {
            Some(path) => classify_local_remote(path),
            None => CheckStatus::NotApplicable {
                why: "This site has no local clone configured.".into(),
            },
        },
        // Checks that need the agent, GitHub, or the server run elsewhere.
        _ => CheckStatus::NotApplicable {
            why: format!("{} runs against a live environment.", check.title()),
        },
    }
}

/// Find an existing private key in `ssh_dir`, if any. Returns the path so the
/// caller can offer to *use* it (never replace it).
pub fn find_local_key(ssh_dir: &Path) -> Option<PathBuf> {
    KEY_CANDIDATES
        .iter()
        .map(|name| ssh_dir.join(name))
        .find(|p| p.exists())
}

/// Build the C1 key-generation fix, honouring the by-construction guarantee: it is
/// `None` when a key already exists, so no code path can reach `ssh-keygen`.
pub fn local_key_fix(ssh_dir: &Path) -> Option<Fix> {
    let existing = find_local_key(ssh_dir);
    let target = ssh_dir.join("id_ed25519");
    key_generation_fix(
        existing.as_ref().map(|_| "present"),
        &target.to_string_lossy(),
    )
}

/// Classify a repository's `origin` remote as SSH, HTTPS (→ wizard), or other.
fn classify_local_remote(repo_path: &Path) -> CheckStatus {
    let repo = match git2::Repository::open(repo_path) {
        Ok(r) => r,
        Err(e) => {
            return CheckStatus::Fail {
                what_is_wrong: format!("Could not open the local repository: {e}"),
            }
        }
    };
    let url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(str::to_string));
    match url {
        Some(url) => match classify_remote(&url) {
            RemoteKind::Ssh => CheckStatus::Pass,
            RemoteKind::Https => CheckStatus::Fail {
                what_is_wrong: format!("The remote `{url}` uses HTTPS, which needs a token."),
            },
            RemoteKind::Other => CheckStatus::Fail {
                what_is_wrong: format!("The remote `{url}` is neither SSH nor HTTPS."),
            },
        },
        None => CheckStatus::Fail {
            what_is_wrong: "This repository has no `origin` remote.".into(),
        },
    }
}

/// Apply a previewed fix that Popush can perform directly (: preview shown
/// first by the UI). Remote conversion runs through `git2`, never a shell, so
/// the URL is set structurally, not string-spliced.
pub fn apply_fix(fix: &Fix, repo_path: Option<&Path>) -> Result<(), String> {
    match fix {
        Fix::ConvertRemote { preview } => {
            let repo_path = repo_path.ok_or("no local repository for remote conversion")?;
            // The preview command is `git remote set-url <name> <new-url>`; take the
            // final token as the new URL and set it via git2.
            let new_url = preview
                .command
                .split_whitespace()
                .last()
                .ok_or("malformed conversion preview")?;
            let repo = git2::Repository::open(repo_path).map_err(|e| e.to_string())?;
            repo.remote_set_url("origin", new_url)
                .map_err(|e| e.to_string())
        }
        // Key generation is applied by the caller via a previewed `ssh-keygen`
        // invocation, guarded so it is unreachable when a key exists. It is
        // not performed here because it runs a process, not a git2 call.
        Fix::GenerateLocalKey { .. } => {
            Err("key generation is applied through its previewed command".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp() -> PathBuf {
        // A unique-enough scratch dir under the system temp; created fresh.
        let base = std::env::temp_dir().join(format!("popush-wiz-{}", std::process::id()));
        let _ = fs::create_dir_all(&base);
        base
    }

    #[test]
    fn local_key_detected_when_present() {
        let dir = tmp().join("has-key");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("id_ed25519"), "x").unwrap();
        assert!(find_local_key(&dir).is_some());
        assert_eq!(
            run_local_check(Check::LocalKeyExists, &dir, None),
            CheckStatus::Pass
        );
    }

    #[test]
    fn missing_local_key_fails_and_offers_generation() {
        let dir = tmp().join("no-key");
        fs::create_dir_all(&dir).unwrap();
        assert!(find_local_key(&dir).is_none());
        // A key does not exist, so the generation fix IS offered.
        assert!(local_key_fix(&dir).is_some());
    }

    #[test]
    fn key_generation_impossible_when_key_exists() {
        let dir = tmp().join("guard");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("id_ed25519"), "x").unwrap();
        // By construction, no generation fix is offered.
        assert!(local_key_fix(&dir).is_none());
    }
}
