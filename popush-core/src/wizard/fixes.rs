use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::git::remote::{https_to_ssh, set_url_preview};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct FixPreview {
    pub command: String,
    pub description: String,
    pub undo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "fix", rename_all = "snake_case")]
pub enum Fix {
    GenerateLocalKey { preview: FixPreview },
    ConvertRemote { preview: FixPreview },
}

pub fn key_generation_fix(existing_key: Option<&str>, key_path: &str) -> Option<Fix> {
    if existing_key.is_some() {
        return None;
    }
    let preview = FixPreview {
        command: format!("ssh-keygen -t ed25519 -f {key_path} -N \"\" -C \"popush\""),
        description:
            "Create a new ed25519 SSH key. A passphrase is more secure but requires ssh-agent."
                .into(),
        undo: Some(format!("rm {key_path} {key_path}.pub")),
    };
    Some(Fix::GenerateLocalKey { preview })
}

pub fn remote_conversion_fix(remote_name: &str, current_url: &str) -> Option<Fix> {
    let new_url = https_to_ssh(current_url)?;
    let preview = FixPreview {
        command: set_url_preview(remote_name, &new_url),
        description: format!("Point `{remote_name}` at GitHub over SSH instead of HTTPS."),
        undo: Some(set_url_preview(remote_name, current_url)),
    };
    Some(Fix::ConvertRemote { preview })
}

pub fn personal_key_warning(server_key_comment: &str) -> Option<String> {
    let c = server_key_comment.to_lowercase();
    let looks_like_deploy_key = c.contains("deploy") || c.contains("popush");
    if looks_like_deploy_key {
        None
    } else {
        Some(
            "This server has a personal SSH key on it. A compromised server would then have your \
             full GitHub access. Use a read-only deploy key scoped to one repository instead."
                .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_generation_is_impossible_when_a_key_exists() {
        assert_eq!(
            key_generation_fix(Some("ssh-ed25519 AAAA..."), "~/.ssh/id_ed25519"),
            None
        );
    }

    #[test]
    fn key_generation_is_offered_only_when_absent_and_is_reversible() {
        let fix = key_generation_fix(None, "~/.ssh/id_ed25519").expect("should offer");
        match fix {
            Fix::GenerateLocalKey { preview } => {
                assert!(preview.command.contains("ssh-keygen -t ed25519"));
                assert!(preview.undo.is_some(), "must show how to undo (D13)");
            }
            _ => panic!("wrong fix"),
        }
    }

    #[test]
    fn remote_conversion_previews_and_is_reversible() {
        let fix = remote_conversion_fix("origin", "https://github.com/twostep/popush.git")
            .expect("should convert");
        match fix {
            Fix::ConvertRemote { preview } => {
                assert!(preview
                    .command
                    .contains("git@github.com:twostep/popush.git"));
                let undo = preview.undo.expect("reversible");
                assert!(undo.contains("https://github.com/twostep/popush.git"));
            }
            _ => panic!("wrong fix"),
        }
    }

    #[test]
    fn remote_conversion_declined_for_non_https() {
        assert_eq!(
            remote_conversion_fix("origin", "git@github.com:twostep/popush.git"),
            None
        );
    }

    #[test]
    fn personal_key_on_server_warns() {
        assert!(personal_key_warning("david@laptop").is_some());
    }

    #[test]
    fn deploy_key_on_server_does_not_warn() {
        assert!(personal_key_warning("popush-deploy-key").is_none());
        assert!(personal_key_warning("deploy@sterling").is_none());
    }
}
