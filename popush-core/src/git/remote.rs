//! Git remote URL classification and HTTPS→SSH conversion.
//!
//! GitHub removed password auth for git-over-HTTPS in August 2021, so any HTTPS
//! push is a token in disguise. Popush does not collect tokens; instead it detects
//! an HTTPS remote and offers to convert it to SSH (wizard C4/C7), which removes
//! the whole category of problem. This module must recognise the many forms a
//! GitHub URL takes.

/// The transport a remote URL uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteKind {
    /// `git@host:owner/repo.git` or `ssh://git@host/owner/repo.git`.
    Ssh,
    /// `https://host/owner/repo.git`.
    Https,
    /// Anything else (git://, file paths, unrecognised).
    Other,
}

/// Classify a remote URL by transport.
pub fn classify_remote(url: &str) -> RemoteKind {
    let u = url.trim();
    if u.starts_with("https://") || u.starts_with("http://") {
        RemoteKind::Https
    } else if u.starts_with("ssh://") || is_scp_like(u) {
        RemoteKind::Ssh
    } else {
        RemoteKind::Other
    }
}

/// The scp-like SSH short form: `user@host:path`, with a colon that is not part of
/// a `scheme://`. This is the most common GitHub SSH form (`git@github.com:o/r`).
fn is_scp_like(u: &str) -> bool {
    if u.contains("://") {
        return false;
    }
    // Must have a `user@host:` shape: an `@`, then a `:` after it, and no `/`
    // before that colon (which would make it a path).
    let Some(at) = u.find('@') else {
        return false;
    };
    let after_at = &u[at + 1..];
    match after_at.find(':') {
        Some(colon) => !after_at[..colon].contains('/'),
        None => false,
    }
}

/// Convert an HTTPS GitHub-style URL to its SSH scp-like equivalent (C4/C7).
///
/// `https://github.com/owner/repo.git` → `git@github.com:owner/repo.git`.
/// Returns `None` if the URL is not HTTPS or cannot be parsed into host + path,
/// so the caller never presents a bogus conversion.
pub fn https_to_ssh(url: &str) -> Option<String> {
    let u = url.trim();
    let rest = u
        .strip_prefix("https://")
        .or_else(|| u.strip_prefix("http://"))?;

    // Strip any embedded credentials (`user:token@host`), they must never carry
    // into the SSH URL, and their presence is exactly why we convert.
    let rest = match rest.split_once('@') {
        Some((_creds, after)) => after,
        None => rest,
    };

    let (host, path) = rest.split_once('/')?;
    if host.is_empty() || path.is_empty() {
        return None;
    }
    // Normalise: ensure a single `.git` suffix is preserved if present, and no
    // trailing slash. We keep the path exactly, only trimming a leading slash.
    let path = path.trim_end_matches('/');
    if path.is_empty() {
        return None;
    }
    Some(format!("git@{host}:{path}"))
}

/// Build the `git remote set-url` command shown before applying (C4/C7). Returned
/// as the display string only; the binary runs it via `git2`, not a shell, so
/// this exists for the preview.
pub fn set_url_preview(remote_name: &str, new_url: &str) -> String {
    format!("git remote set-url {remote_name} {new_url}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_the_common_forms() {
        assert_eq!(
            classify_remote("git@github.com:twostep/popush.git"),
            RemoteKind::Ssh
        );
        assert_eq!(
            classify_remote("ssh://git@github.com/twostep/popush.git"),
            RemoteKind::Ssh
        );
        assert_eq!(
            classify_remote("https://github.com/twostep/popush.git"),
            RemoteKind::Https
        );
        assert_eq!(
            classify_remote("https://token@github.com/twostep/popush.git"),
            RemoteKind::Https
        );
        assert_eq!(
            classify_remote("git://github.com/x/y.git"),
            RemoteKind::Other
        );
        assert_eq!(classify_remote("/srv/local/repo.git"), RemoteKind::Other);
    }

    #[test]
    fn converts_https_to_ssh_scp_form() {
        assert_eq!(
            https_to_ssh("https://github.com/twostep/popush.git").as_deref(),
            Some("git@github.com:twostep/popush.git")
        );
    }

    #[test]
    fn converts_without_git_suffix() {
        assert_eq!(
            https_to_ssh("https://github.com/twostep/popush").as_deref(),
            Some("git@github.com:twostep/popush")
        );
    }

    #[test]
    fn strips_embedded_credentials_on_conversion() {
        // The token must not survive into the SSH URL.
        let out = https_to_ssh("https://ghp_secret@github.com/twostep/popush.git").unwrap();
        assert_eq!(out, "git@github.com:twostep/popush.git");
        assert!(!out.contains("ghp_secret"));
    }

    #[test]
    fn handles_self_hosted_host() {
        assert_eq!(
            https_to_ssh("https://git.example.com/group/sub/repo.git").as_deref(),
            Some("git@git.example.com:group/sub/repo.git")
        );
    }

    #[test]
    fn trailing_slash_is_trimmed() {
        assert_eq!(
            https_to_ssh("https://github.com/twostep/popush.git/").as_deref(),
            Some("git@github.com:twostep/popush.git")
        );
    }

    #[test]
    fn non_https_returns_none() {
        assert_eq!(https_to_ssh("git@github.com:twostep/popush.git"), None);
        assert_eq!(https_to_ssh("https://github.com/"), None);
        assert_eq!(https_to_ssh("https://github.com"), None);
    }

    #[test]
    fn set_url_preview_is_readable() {
        assert_eq!(
            set_url_preview("origin", "git@github.com:o/r.git"),
            "git remote set-url origin git@github.com:o/r.git"
        );
    }
}
