pub const REDACTED: &str = "[redacted]";

pub fn redact_line(input: &str) -> String {
    let mut out = input.to_string();
    out = redact_private_key(&out);
    out = redact_authorization(&out);
    out = redact_secret_kv(&out);
    out = redact_tokens(&out);
    out
}

pub fn is_private_key_begin(line: &str) -> bool {
    line.contains("-----BEGIN") && line.contains("PRIVATE KEY")
}

pub fn is_private_key_end(line: &str) -> bool {
    line.contains("-----END") && line.contains("PRIVATE KEY")
}

fn find_ci_ascii(haystack: &str, needle: &str, from: usize) -> Option<usize> {
    let hay = haystack.as_bytes();
    let ndl = needle.as_bytes();
    if ndl.is_empty() || hay.len() < ndl.len() {
        return None;
    }
    (from..=hay.len() - ndl.len()).find(|&i| {
        hay[i..i + ndl.len()]
            .iter()
            .zip(ndl)
            .all(|(h, n)| h.to_ascii_lowercase() == *n)
    })
}

fn redact_secret_kv(input: &str) -> String {
    const KEYS: [&str; 4] = ["passphrase", "passwd", "password", "pwd"];
    let mut result = String::with_capacity(input.len());
    let mut i = 0;
    while i <= input.len() {
        let mut best: Option<(usize, usize)> = None;
        for key in KEYS {
            if let Some(pos) = find_ci_ascii(input, key, i) {
                if best.map(|(b, _)| pos < b).unwrap_or(true) {
                    best = Some((pos, key.len()));
                }
            }
        }
        let Some((pos, klen)) = best else {
            result.push_str(&input[i..]);
            break;
        };
        let after_key = pos + klen;
        let sep_rel = input[after_key..].find([':', '=']);
        let sep = match sep_rel {
            Some(sr)
                if input[after_key..after_key + sr]
                    .chars()
                    .all(|c| c == ' ' || c == '\t') =>
            {
                after_key + sr
            }
            _ => {
                result.push_str(&input[i..after_key]);
                i = after_key;
                continue;
            }
        };
        let mut cut = sep + 1;
        if input[cut..].starts_with(' ') {
            cut += 1;
        }
        let val_len = input[cut..]
            .find(char::is_whitespace)
            .unwrap_or(input[cut..].len());
        result.push_str(&input[i..cut]);
        if val_len > 0 {
            result.push_str(REDACTED);
        }
        i = cut + val_len;
    }
    result
}

fn redact_tokens(input: &str) -> String {
    const PREFIXES: [&str; 5] = ["ghp_", "github_pat_", "gho_", "ghs_", "ghr_"];
    let mut result = String::with_capacity(input.len());
    let mut rest = input;
    'outer: loop {
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
        let after = &rest[idx + prefix.len()..];
        let body_len = after
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(after.len());
        result.push_str(REDACTED);
        rest = &after[body_len..];
    }
    result
}

fn redact_authorization(input: &str) -> String {
    let lower = input.to_lowercase();
    let Some(pos) = lower.find("authorization") else {
        return input.to_string();
    };
    let after_key = pos + "authorization".len();
    let sep_rel = input[after_key..].find([':', '=']);
    let Some(sep_rel) = sep_rel else {
        return input.to_string();
    };
    let sep = after_key + sep_rel;
    let mut cut = sep + 1;
    if input[cut..].starts_with(' ') {
        cut += 1;
    }
    format!("{}{}", &input[..cut], REDACTED)
}

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
        let line = "deployed a3f9c21 to production";
        assert_eq!(redact_line(line), line);
    }

    #[test]
    fn redacts_password_key_value() {
        assert_eq!(redact_line("password=hunter2"), "password=[redacted]");
        assert_eq!(
            redact_line("db passphrase: s3cr3t rest"),
            "db passphrase: [redacted] rest"
        );
        assert_eq!(redact_line("PWD = topsecret"), "PWD = [redacted]");
    }

    #[test]
    fn redacts_multiple_password_values() {
        assert_eq!(
            redact_line("password=a and passwd=b"),
            "password=[redacted] and passwd=[redacted]"
        );
    }

    #[test]
    fn password_prose_without_a_value_is_untouched() {
        let line = "the user requested a password reset by email";
        assert_eq!(redact_line(line), line);
    }

    #[test]
    fn private_key_boundaries_are_detected() {
        assert!(is_private_key_begin("-----BEGIN OPENSSH PRIVATE KEY-----"));
        assert!(is_private_key_begin("  -----BEGIN RSA PRIVATE KEY-----"));
        assert!(is_private_key_end("-----END OPENSSH PRIVATE KEY-----"));
        assert!(!is_private_key_begin("just a normal line"));
        assert!(!is_private_key_end("-----BEGIN OPENSSH PRIVATE KEY-----"));
    }
}
