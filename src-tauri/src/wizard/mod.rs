use std::path::{Path, PathBuf};

use popush_core::git::remote::classify_remote;
use popush_core::git::RemoteKind;
use popush_core::wizard::fixes::key_generation_fix;
use popush_core::wizard::{Check, CheckStatus, Fix};

const KEY_CANDIDATES: [&str; 3] = ["id_ed25519", "id_ecdsa", "id_rsa"];

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
        _ => CheckStatus::NotApplicable {
            why: format!("{} runs against a live environment.", check.title()),
        },
    }
}

pub fn find_local_key(ssh_dir: &Path) -> Option<PathBuf> {
    KEY_CANDIDATES
        .iter()
        .map(|name| ssh_dir.join(name))
        .find(|p| p.exists())
}

pub fn local_key_fix(ssh_dir: &Path) -> Option<Fix> {
    let existing = find_local_key(ssh_dir);
    let target = ssh_dir.join("id_ed25519");
    key_generation_fix(
        existing.as_ref().map(|_| "present"),
        &target.to_string_lossy(),
    )
}

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

pub fn apply_fix(fix: &Fix, repo_path: Option<&Path>) -> Result<(), String> {
    match fix {
        Fix::ConvertRemote { .. } => {
            let repo_path = repo_path.ok_or("no local repository for remote conversion")?;
            let repo = git2::Repository::open(repo_path).map_err(|e| e.to_string())?;
            let current = repo
                .find_remote("origin")
                .ok()
                .and_then(|r| r.url().map(str::to_string))
                .ok_or("repository has no origin remote to convert")?;
            if classify_remote(&current) != RemoteKind::Https {
                return Err("origin is not an HTTPS remote; nothing to convert".into());
            }
            let new_url = popush_core::git::remote::https_to_ssh(&current)
                .ok_or("could not convert the origin URL to SSH")?;
            repo.remote_set_url("origin", &new_url)
                .map_err(|e| e.to_string())
        }
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
        assert!(local_key_fix(&dir).is_some());
    }

    #[test]
    fn key_generation_impossible_when_key_exists() {
        let dir = tmp().join("guard");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("id_ed25519"), "x").unwrap();
        assert!(local_key_fix(&dir).is_none());
    }
}
