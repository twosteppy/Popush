#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteKind {
    Ssh,
    Https,
    Other,
}

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

fn is_scp_like(u: &str) -> bool {
    if u.contains("://") {
        return false;
    }
    let Some(at) = u.find('@') else {
        return false;
    };
    let after_at = &u[at + 1..];
    match after_at.find(':') {
        Some(colon) => !after_at[..colon].contains('/'),
        None => false,
    }
}

pub fn https_to_ssh(url: &str) -> Option<String> {
    let u = url.trim();
    let rest = u
        .strip_prefix("https://")
        .or_else(|| u.strip_prefix("http://"))?;

    let rest = match rest.split_once('@') {
        Some((_creds, after)) => after,
        None => rest,
    };

    let (host, path) = rest.split_once('/')?;
    if host.is_empty() || path.is_empty() {
        return None;
    }
    let path = path.trim_end_matches('/');
    if path.is_empty() {
        return None;
    }
    Some(format!("git@{host}:{path}"))
}

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
