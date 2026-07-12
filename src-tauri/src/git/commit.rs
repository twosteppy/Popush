//! Local commit and push via `git2` (§10.1, §10.3), with agent-based credentials
//! (§10.3). The refusals and URL classification are decided by `popush_core`; this
//! layer performs the libgit2 calls (D14).
//!
//! Push credentials use the **same agent delegation** as SSH (§10.3): `git2`'s
//! credential callback asks `ssh-agent`. Popush never collects a token; if the
//! remote is HTTPS the caller routes to the wizard rather than reaching here.

use std::path::{Path, PathBuf};

use popush_core::error::GitError;
use popush_core::git::remote::classify_remote;
use popush_core::git::RemoteKind;

/// Stage the given paths and commit them with `message`. Returns the new commit's
/// short SHA. Refuses on merge conflicts and detached HEAD with the exact
/// `popush_core` errors (§10.2).
pub fn stage_and_commit(
    repo_path: &Path,
    message: &str,
    files: &[PathBuf],
) -> Result<String, GitError> {
    let repo = open(repo_path)?;

    // Refuse in the states Popush does not handle (§10.2).
    if repo.state() != git2::RepositoryState::Clean {
        let conflicted = conflicted_paths(&repo)?;
        if !conflicted.is_empty() {
            return Err(GitError::MergeConflicts {
                count: conflicted.len(),
                files: conflicted,
            });
        }
    }
    let head = repo.head().map_err(op)?;
    if !head.is_branch() {
        return Err(GitError::DetachedHead);
    }

    // Stage the selected files.
    let mut index = repo.index().map_err(op)?;
    for file in files {
        index.add_path(file).map_err(op)?;
    }
    index.write().map_err(op)?;
    let tree_oid = index.write_tree().map_err(op)?;
    let tree = repo.find_tree(tree_oid).map_err(op)?;

    // Build the commit on top of HEAD.
    let parent = head.peel_to_commit().map_err(op)?;
    let signature = repo.signature().map_err(op)?;
    let commit_oid = repo
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent],
        )
        .map_err(op)?;

    Ok(commit_oid.to_string().chars().take(7).collect())
}

/// Push `branch` to `remote_name`. Refuses HTTPS remotes (routes to the wizard,
/// §10.3) and classifies rejection reasons (§12.4).
pub fn push(repo_path: &Path, remote_name: &str, branch: &str) -> Result<(), GitError> {
    let repo = open(repo_path)?;
    let mut remote = repo.find_remote(remote_name).map_err(op)?;

    let url = remote.url().unwrap_or_default().to_string();
    if classify_remote(&url) == RemoteKind::Https {
        return Err(GitError::HttpsRemote { url });
    }

    // The server's per-reference rejection message arrives in a callback that
    // borrows for the whole push; a RefCell lets us read it back afterwards. The
    // callbacks/options are scoped so their borrow of `push_error` ends before we
    // read it.
    let push_error: std::cell::RefCell<Option<GitError>> = std::cell::RefCell::new(None);
    let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");
    {
        let mut callbacks = git2::RemoteCallbacks::new();
        // Agent delegation (§10.3): let ssh-agent sign. Popush never sees the key.
        callbacks.credentials(|_url, username, allowed| {
            if allowed.contains(git2::CredentialType::SSH_KEY) {
                git2::Cred::ssh_key_from_agent(username.unwrap_or("git"))
            } else {
                Err(git2::Error::from_str("only ssh-agent auth is supported"))
            }
        });
        callbacks.push_update_reference(|_refname, status| {
            if let Some(msg) = status {
                *push_error.borrow_mut() = Some(classify_push_rejection(msg));
            }
            Ok(())
        });

        let mut options = git2::PushOptions::new();
        options.remote_callbacks(callbacks);

        remote
            .push(&[&refspec], Some(&mut options))
            .map_err(|e| classify_push_rejection(&e.to_string()))?;
    }

    if let Some(err) = push_error.into_inner() {
        return Err(err);
    }
    Ok(())
}

/// Map a push rejection message to a structured error (§12.4).
fn classify_push_rejection(msg: &str) -> GitError {
    let m = msg.to_lowercase();
    if m.contains("non-fast-forward") || m.contains("fetch first") || m.contains("behind") {
        GitError::PushRejectedNonFastForward
    } else if m.contains("permission")
        || m.contains("denied")
        || m.contains("authentication")
        || m.contains("publickey")
    {
        GitError::PushRejectedPermission
    } else {
        GitError::Operation {
            detail: msg.to_string(),
        }
    }
}

fn conflicted_paths(repo: &git2::Repository) -> Result<Vec<PathBuf>, GitError> {
    let index = repo.index().map_err(op)?;
    let mut out = Vec::new();
    if let Ok(conflicts) = index.conflicts() {
        for c in conflicts.flatten() {
            if let Some(entry) = c.our.or(c.their) {
                out.push(PathBuf::from(
                    String::from_utf8_lossy(&entry.path).into_owned(),
                ));
            }
        }
    }
    Ok(out)
}

fn open(path: &Path) -> Result<git2::Repository, GitError> {
    git2::Repository::open(path).map_err(op)
}

fn op(e: git2::Error) -> GitError {
    GitError::Operation {
        detail: e.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_non_fast_forward() {
        assert_eq!(
            classify_push_rejection(
                "Updates were rejected because the remote contains work (non-fast-forward)"
            ),
            GitError::PushRejectedNonFastForward
        );
    }

    #[test]
    fn classifies_permission_denied() {
        assert_eq!(
            classify_push_rejection("ERROR: Permission to x denied to y (publickey)"),
            GitError::PushRejectedPermission
        );
    }

    #[test]
    fn classifies_unknown_as_operation() {
        assert!(matches!(
            classify_push_rejection("some other libgit2 message"),
            GitError::Operation { .. }
        ));
    }
}
