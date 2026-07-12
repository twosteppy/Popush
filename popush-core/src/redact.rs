//! Credential redaction for logs (§17.3, defence in §17.2).
//!
//! The `tracing` layer must never write key material, tokens, or `Authorization`
//! headers, even though logs are local-only and never transmitted (D3). Redaction
//! is applied before a line is written. The transformation is pure and tested
//! here; the binary installs it as a formatting layer.
//!
//! This errs toward over-redaction: a false positive replaces a harmless string,
//! which is preferable to a false negative that writes a secret.

/// The replacement shown in place of a redacted secret.
const REDACTED: &str = "[redacted]";

/// Redact secrets from a single log line.
///
/// Covered patterns:
/// * GitHub tokens (`ghp_…`, `github_pat_…`, `gho_…`, `ghs_…`, `ghr_…`).
/// * `Authorization: …` / `Authorization=…` header values.
/// * Inline private-key material (`-----BEGIN … PRIVATE KEY-----` onward).
pub fn redact_line(input: &str) -> String {
    let mut out = input.to_string();
    out = redact_private_key(&out);
    out = redact_authorization(&out);
    out = redact_tokens(&out);
    out
}

/// Replace any run that looks like a GitHub token with the placeholder. Tokens are
/// a known prefix followed by base62-ish characters.
fn redact_tokens(input: &str) -> String {
    const PREFIXES: [&str; 5] = ["ghp_", "github_pat_", "gho_", "ghs_", "ghr_"];
    let mut result = String::with_capacity(input.len());
    let mut rest = input;
    'outer: loop {
        // Find the earliest occurrence of any prefix.
        let mut best: Option<(usize, &str)> = None;
        for p in PREFIXES {
            if let Some(idx) = rest.find(p) {
                if best.map(|(b, _)| idx < b).unwrap_or(true) {
                    best = Some((idx, p));
                }
            }
        }
        let Some((idx, prefix)) = best else {
            result.push_str(rest);
            break 'outer;
        };
        result.push_str(&rest[..idx]);
        // Consume the prefix and the following token body (word characters, `_`).
        let after = &rest[idx + prefix.len()..];
        let body_len = after
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(after.len());
        result.push_str(REDACTED);
        rest = &after[body_len..];
    }
    result
}

/// Redact the value after an `Authorization` header key, keeping the key visible.
fn redact_authorization(input: &str) -> String {
    let lower = input.to_lowercase();
    let Some(pos) = lower.find("authorization") else {
        return input.to_string();
    };
    // Find the separator (`:` or `=`) after the key, then redact to end of line.
    let after_key = pos + "authorization".len();
    let sep_rel = input[after_key..].find([':', '=']);
    let Some(sep_rel) = sep_rel else {
        return input.to_string();
    };
    let sep = after_key + sep_rel;
    // Preserve up to and including the separator and any single following space.
    let mut cut = sep + 1;
    if input[cut..].starts_with(' ') {
        cut += 1;
    }
    format!("{}{}", &input[..cut], REDACTED)
}

/// Redact an inline private-key block from its header onward.
fn redact_private_key(input: &str) -> String {
    if let Some(idx) = input.find("-----BEGIN") {
        if input[idx..].contains("PRIVATE KEY") {
            return format!("{}{}", &input[..idx], REDACTED);
        }
    }
    input.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_github_classic_token() {
        let out = redact_line("cloning with ghp_ABCdef123456 done");
        assert_eq!(out, "cloning with [redacted] done");
        assert!(!out.contains("ghp_ABCdef123456"));
    }

    #[test]
    fn redacts_fine_grained_token() {
        let out = redact_line("token=github_pat_11ABCDEF_xyz rest");
        assert!(!out.contains("github_pat_11ABCDEF_xyz"));
        assert!(out.contains("[redacted]"));
    }

    #[test]
    fn redacts_authorization_header() {
        let out = redact_line("Authorization: Bearer sk-secret-value");
        assert_eq!(out, "Authorization: [redacted]");
    }

    #[test]
    fn redacts_authorization_with_equals() {
        let out = redact_line("header Authorization=token123");
        assert!(out.ends_with("[redacted]"));
        assert!(!out.contains("token123"));
    }

    #[test]
    fn redacts_inline_private_key() {
        let out = redact_line("key: -----BEGIN OPENSSH PRIVATE KEY-----MIIB");
        assert_eq!(out, "key: [redacted]");
        assert!(!out.contains("MIIB"));
    }

    #[test]
    fn leaves_ordinary_lines_untouched() {
        let line = "cd /srv/site && docker compose up -d";
        assert_eq!(redact_line(line), line);
    }

    #[test]
    fn redacts_multiple_tokens_in_one_line() {
        let out = redact_line("a ghp_one b ghp_two c");
        assert_eq!(out, "a [redacted] b [redacted] c");
    }

    #[test]
    fn a_sha_is_not_mistaken_for_a_token() {
        // A commit SHA has no token prefix and must survive.
        let line = "deployed a3f9c21 to production";
        assert_eq!(redact_line(line), line);
    }
}
