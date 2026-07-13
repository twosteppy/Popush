use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownHost {
    pub host: String,
    pub key_type: String,
    pub key_base64: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum HostKeyDecision {
    Trusted,
    Unknown { fingerprint: String },
    Mismatch { expected: String, got: String },
}

pub struct HostKeyVerifier<'a> {
    known: &'a [KnownHost],
}

impl<'a> HostKeyVerifier<'a> {
    pub fn new(known: &'a [KnownHost]) -> Self {
        Self { known }
    }

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

        if entries_for_host
            .iter()
            .any(|k| k.key_type == key_type && k.key_base64 == presented_base64)
        {
            return HostKeyDecision::Trusted;
        }

        if let Some(expected) = entries_for_host.iter().find(|k| k.key_type == key_type) {
            return HostKeyDecision::Mismatch {
                expected: expected.key_base64.clone(),
                got: presented_base64.to_string(),
            };
        }

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
