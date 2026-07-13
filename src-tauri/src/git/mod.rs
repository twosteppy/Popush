pub mod commit;

pub use commit::{push, stage_and_commit};

use std::path::Path;

use popush_core::config::{ChangeKind, ChangedFile, GitStatus};
use popush_core::error::GitError;
use popush_core::git::remote::classify_remote;
use popush_core::git::RemoteKind;

pub fn status(path: &Path, remote_name: &str) -> Result<GitStatus, GitError> {
    let repo = git2::Repository::open(path).map_err(|e| GitError::Operation {
        detail: e.to_string(),
    })?;

    let head = repo.head().map_err(|e| GitError::Operation {
        detail: e.to_string(),
    })?;
    if !head.is_branch() {
        return Err(GitError::DetachedHead);
    }
    let branch = head.shorthand().unwrap_or("HEAD").to_string();

    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(true);
    let statuses = repo
        .statuses(Some(&mut opts))
        .map_err(|e| GitError::Operation {
            detail: e.to_string(),
        })?;

    let mut changed_files = Vec::new();
    let mut has_conflicts = false;
    for entry in statuses.iter() {
        let s = entry.status();
        if s.is_conflicted() {
            has_conflicts = true;
        }
        if let Some(path) = entry.path() {
            if let Some(cf) = classify_status(path, s) {
                changed_files.push(cf);
            }
        }
    }

    let (ahead, behind) = ahead_behind(&repo).unwrap_or((0, 0));

    let (remote_url, remote_is_ssh) = match repo.find_remote(remote_name) {
        Ok(remote) => {
            let url = remote.url().unwrap_or_default().to_string();
            let is_ssh = classify_remote(&url) == RemoteKind::Ssh;
            (url, is_ssh)
        }
        Err(_) => (String::new(), false),
    };

    Ok(GitStatus {
        branch,
        ahead,
        behind,
        changed_files,
        has_conflicts,
        remote_url,
        remote_is_ssh,
    })
}

fn classify_status(path: &str, s: git2::Status) -> Option<ChangedFile> {
    let staged = s.intersects(
        git2::Status::INDEX_NEW
            | git2::Status::INDEX_MODIFIED
            | git2::Status::INDEX_DELETED
            | git2::Status::INDEX_RENAMED,
    );
    let change = if s.intersects(git2::Status::WT_NEW) {
        ChangeKind::Untracked
    } else if s.intersects(git2::Status::INDEX_NEW) {
        ChangeKind::Added
    } else if s.intersects(git2::Status::WT_DELETED | git2::Status::INDEX_DELETED) {
        ChangeKind::Deleted
    } else if s.intersects(git2::Status::WT_RENAMED | git2::Status::INDEX_RENAMED) {
        ChangeKind::Renamed
    } else if s.intersects(git2::Status::WT_MODIFIED | git2::Status::INDEX_MODIFIED) {
        ChangeKind::Modified
    } else {
        return None;
    };
    Some(ChangedFile {
        path: path.into(),
        change,
        staged,
    })
}

fn ahead_behind(repo: &git2::Repository) -> Option<(usize, usize)> {
    let head = repo.head().ok()?;
    let local_oid = head.target()?;
    let branch_name = head.shorthand()?;
    let upstream = repo
        .find_branch(branch_name, git2::BranchType::Local)
        .ok()?
        .upstream()
        .ok()?;
    let upstream_oid = upstream.get().target()?;
    repo.graph_ahead_behind(local_oid, upstream_oid).ok()
}
