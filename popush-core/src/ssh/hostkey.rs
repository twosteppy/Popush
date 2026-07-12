//! Host-key verification decision logic (§8.3).
//!
//! The rule is exactly `ssh`'s: a known, matching key connects; an unknown host
//! is presented for the user to verify by fingerprint and **never auto-accepted**;
//! a known host whose key has **changed** is refused, because that is the
//! signature of a man-in-the-middle. The refusal is not one-click dismissible in
//! the UI (§8.3 gate); this module only produces the decision, the UI enforces
//! the friction.
//!
//! This is decision logic over already-parsed data, so it is pure and unit-tested
//! here. Reading `~/.ssh/known_hosts` and computing fingerprints from wire bytes
//! happens in the binary layer.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One entry from `known_hosts`: a host pattern and the base64 key it is pinned to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownHost {
    /// The host as it appears in `known_hosts` (e.g. `203.0.113.10` or
    /// `[example.com]:2222`). Hashed host lines are matched by the caller before
    /// constructing this.
    pub host: String,
    /// The key type, e.g. `ssh-ed25519`.
    pub key_type: String,
    /// The base64-encoded public key blob.
    pub key_base64: String,
}

/// The verifier's verdict for a presented key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum HostKeyDecision {
    /// The presented key matches a pinned entry. Connect.
    Trusted,
    /// The host is unknown. Present the fingerprint and ask the user (§8.3).
    Unknown {
        /// The fingerprint to show for verification.
        fingerprint: String,
    },
    /// The host is known but the key changed. Refuse, warn loudly (§8.3).
    Mismatch {
        /// The pinned key Popush expected.
        expected: String,
        /// The key actually presented.
        got: String,
    },
}

/// Verifies a presented key against known entries for a host.
pub struct HostKeyVerifier<'a> {
    known: &'a [KnownHost],
}

impl<'a> HostKeyVerifier<'a> {
    /// Wrap the set of `known_hosts` entries relevant to the connection.
    pub fn new(known: &'a [KnownHost]) -> Self {
        Self { known }
    }

    /// Decide whether to trust `presented` (base64 key blob) for `host`.
    ///
    /// `fingerprint` is the human-readable `SHA256:…` form the caller computes
    /// from the presented key; it is only used in the `Unknown` and `Mismatch`
    /// arms so the UI can show something meaningful.
    pub fn verify(
        &self,
        host: &str,
        key_type: &str,
        presented_base64: &str,
        fingerprint: &str,
    ) -> HostKeyDecision {
        let entries_for_host: Vec<&KnownHost> =
            self.known.iter().filter(|k| k.host == host).collect();

        if entries_for_host.is_empty() {
            return HostKeyDecision::Unknown {
                fingerprint: fingerprint.to_string(),
            };
        }

        // A matching entry of the same key type means trust.
        if entries_for_host
            .iter()
            .any(|k| k.key_type == key_type && k.key_base64 == presented_base64)
        {
            return HostKeyDecision::Trusted;
        }

        // Known host, same key type, different key → mismatch (the MITM signature).
        if let Some(expected) = entries_for_host.iter().find(|k| k.key_type == key_type) {
            return HostKeyDecision::Mismatch {
                expected: expected.key_base64.clone(),
                got: presented_base64.to_string(),
            };
        }

        // Known host but only under other key types: treat the new type as
        // unknown rather than a mismatch, matching OpenSSH behaviour.
        HostKeyDecision::Unknown {
            fingerprint: fingerprint.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kh(host: &str, ty: &str, key: &str) -> KnownHost {
        KnownHost {
            host: host.into(),
            key_type: ty.into(),
            key_base64: key.into(),
        }
    }

    #[test]
    fn matching_key_is_trusted() {
        let known = [kh("h", "ssh-ed25519", "AAAA")];
        let v = HostKeyVerifier::new(&known);
        assert_eq!(
            v.verify("h", "ssh-ed25519", "AAAA", "SHA256:x"),
            HostKeyDecision::Trusted
        );
    }

    #[test]
    fn unknown_host_is_never_auto_accepted() {
        let known = [];
        let v = HostKeyVerifier::new(&known);
        assert_eq!(
            v.verify("h", "ssh-ed25519", "AAAA", "SHA256:x"),
            HostKeyDecision::Unknown {
                fingerprint: "SHA256:x".into()
            }
        );
    }

    #[test]
    fn changed_key_is_a_mismatch_not_a_prompt() {
        let known = [kh("h", "ssh-ed25519", "OLD")];
        let v = HostKeyVerifier::new(&known);
        assert_eq!(
            v.verify("h", "ssh-ed25519", "NEW", "SHA256:x"),
            HostKeyDecision::Mismatch {
                expected: "OLD".into(),
                got: "NEW".into()
            }
        );
    }

    #[test]
    fn new_key_type_for_known_host_is_unknown_not_mismatch() {
        let known = [kh("h", "ssh-rsa", "AAAA")];
        let v = HostKeyVerifier::new(&known);
        assert_eq!(
            v.verify("h", "ssh-ed25519", "BBBB", "SHA256:x"),
            HostKeyDecision::Unknown {
                fingerprint: "SHA256:x".into()
            }
        );
    }
}
