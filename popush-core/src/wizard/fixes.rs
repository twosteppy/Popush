//! Wizard fixes: preview-then-apply, never destructive, always reversible (§11.1,
//! D13). The key invariant — **key generation can never overwrite an existing
//! key** — is enforced *by construction* here (§Phase 6 gate): the fix builder
//! returns `None` when a key already exists, so no code path can reach
//! `ssh-keygen` for a key that is present.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::git::remote::{https_to_ssh, set_url_preview};

/// A previewed fix: exactly what will run and exactly how to undo it (D13).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct FixPreview {
    /// The exact command or change, shown before applying (§11.1 rule 1).
    pub command: String,
    /// Plain-English description of what it does.
    pub description: String,
    /// How to undo it (§11.1 rule 3). `None` only for inherently safe, additive
    /// actions that leave nothing to undo (documented per fix).
    pub undo: Option<String>,
}

/// A fix the wizard can apply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "fix", rename_all = "snake_case")]
pub enum Fix {
    /// Generate a new ed25519 key locally.
    GenerateLocalKey {
        /// The previewed command and its undo.
        preview: FixPreview,
    },
    /// Convert a remote URL from HTTPS to SSH.
    ConvertRemote {
        /// The previewed command and its undo.
        preview: FixPreview,
    },
}

/// Build the key-generation fix (C1).
///
/// **Returns `None` when a key already exists** — this is the by-construction
/// guarantee (D13 rule 2, Phase 6 gate) that Popush can never overwrite a key.
/// The caller cannot obtain a `GenerateLocalKey` fix, and therefore cannot reach
/// `ssh-keygen`, when `existing_key` is `Some`.
pub fn key_generation_fix(existing_key: Option<&str>, key_path: &str) -> Option<Fix> {
    if existing_key.is_some() {
        // A key exists. Offer to *use* it, never replace it. No keygen fix exists.
        return None;
    }
    let preview = FixPreview {
        // `-f` names the target; because we only reach here when no key exists,
        // this can never clobber. ed25519 with no passphrase by default; the UI
        // asks about a passphrase and explains the trade-off (§11.2 C1).
        command: format!("ssh-keygen -t ed25519 -f {key_path} -N \"\" -C \"popush\""),
        description:
            "Create a new ed25519 SSH key. A passphrase is more secure but requires ssh-agent."
                .into(),
        // Undo: delete the freshly created key pair. Safe because it did not exist
        // before this action.
        undo: Some(format!("rm {key_path} {key_path}.pub")),
    };
    Some(Fix::GenerateLocalKey { preview })
}

/// Build the remote-conversion fix (C4/C7). Returns `None` if the URL is not HTTPS
/// or cannot be converted, so the wizard never previews a bogus change.
pub fn remote_conversion_fix(remote_name: &str, current_url: &str) -> Option<Fix> {
    let new_url = https_to_ssh(current_url)?;
    let preview = FixPreview {
        command: set_url_preview(remote_name, &new_url),
        description: format!("Point `{remote_name}` at GitHub over SSH instead of HTTPS."),
        // Fully reversible: set the URL back (§11.2 C4 "Fully reversible, says so").
        undo: Some(set_url_preview(remote_name, current_url)),
    };
    Some(Fix::ConvertRemote { preview })
}

/// Detect a **personal** private key present on the server (§11.3): the wizard
/// must warn about this and never put a personal key on a VPS. A deploy key is
/// scoped and read-only; a personal key grants far more. The heuristic: a private
/// key file whose comment/name is not a deploy-key marker.
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
        // The by-construction guarantee: no fix, therefore no keygen path (D13).
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
