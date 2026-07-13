//! Parsing `~/.ssh/known_hosts` into [`KnownHost`] entries.
//!
//! This is the pure, testable half of host-key verification: turning the on-disk
//! file into the structured entries [`super::hostkey::HostKeyVerifier`] consults.
//! Reading the file from disk happens in the binary; the parsing, and the fiddly
//! host-pattern handling, is here so it can be tested without a filesystem.
//!
//! Hashed host lines (`|1|...`) cannot be matched by plaintext host and are
//! skipped: Popush compares against the plaintext host it is connecting to, and a
//! hashed line would need the salt+HMAC to match. Skipping them means an entry
//! Popush cannot verify is treated as "unknown" rather than silently trusted,
//! which is the safe direction.

use super::hostkey::KnownHost;

/// Parse the contents of a `known_hosts` file, keeping only entries whose host is
/// stored in plaintext. Malformed lines are skipped, not fatal, one bad line must
/// not blind Popush to the rest of the file.
pub fn parse(contents: &str) -> Vec<KnownHost> {
    let mut out = Vec::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // A marker line begins with `@cert-authority` or `@revoked`; those are not
        // plain host-key pins, so skip them.
        if line.starts_with('@') {
            continue;
        }
        // Format: `host[,host2,...] keytype base64 [comment]`.
        let mut parts = line.split_whitespace();
        let Some(hosts) = parts.next() else { continue };
        let Some(key_type) = parts.next() else {
            continue;
        };
        let Some(key_base64) = parts.next() else {
            continue;
        };
        // Hashed host entries cannot be matched against a plaintext host.
        if hosts.starts_with('|') {
            continue;
        }
        // A single line may list several comma-separated host patterns.
        for host in hosts.split(',') {
            let host = normalize_host(host);
            if host.is_empty() {
                continue;
            }
            out.push(KnownHost {
                host,
                key_type: key_type.to_string(),
                key_base64: key_base64.to_string(),
            });
        }
    }
    out
}

/// The `known_hosts` lookup key for a connection, matching OpenSSH exactly: the
/// bare host on the default port 22, and the bracketed `[host]:port` form on any
/// other port. Callers must verify with this key (not the bare host) so a pin
/// made for one port is never reused to trust a *different* service on another
/// port of the same host.
pub fn lookup_key(host: &str, port: u16) -> String {
    if port == 22 {
        host.to_string()
    } else {
        format!("[{host}]:{port}")
    }
}

/// Normalise a host pattern to the form the verifier compares against. OpenSSH
/// writes a non-default port as `[host]:port`; the verifier is given the bare host
/// for the default port and the bracketed form otherwise, so we keep the string as
/// written and only strip an enclosing `[...]` when there is no port.
fn normalize_host(host: &str) -> String {
    // `[example.com]:2222` stays as-is (the caller builds the same bracket form).
    // `[example.com]` with no port unwraps to `example.com`.
    if let Some(inner) = host.strip_prefix('[') {
        if let Some(closing) = inner.strip_suffix(']') {
            return closing.to_string();
        }
        // `[host]:port`, keep the whole pattern.
        return host.to_string();
    }
    host.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_simple_entry() {
        let entries = parse("203.0.113.10 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].host, "203.0.113.10");
        assert_eq!(entries[0].key_type, "ssh-ed25519");
        assert_eq!(entries[0].key_base64, "AAAAC3NzaC1lZDI1NTE5");
    }

    #[test]
    fn skips_comments_and_blank_lines() {
        let entries = parse("# a comment\n\n   \nhost ssh-rsa AAAA");
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn splits_comma_separated_hosts() {
        let entries = parse("host1,host2,192.0.2.1 ssh-ed25519 AAAA");
        let hosts: Vec<&str> = entries.iter().map(|e| e.host.as_str()).collect();
        assert_eq!(hosts, vec!["host1", "host2", "192.0.2.1"]);
    }

    #[test]
    fn skips_hashed_host_lines() {
        // A |1| line cannot be matched against a plaintext host, so it is skipped
        // rather than trusted (the safe direction).
        let entries = parse("|1|abc=|def= ssh-ed25519 AAAA\nplain ssh-ed25519 BBBB");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].host, "plain");
    }

    #[test]
    fn skips_marker_lines() {
        let entries = parse("@cert-authority *.example.com ssh-ed25519 AAAA");
        assert!(entries.is_empty());
    }

    #[test]
    fn unwraps_bracketed_host_without_port() {
        let entries = parse("[example.com] ssh-ed25519 AAAA");
        assert_eq!(entries[0].host, "example.com");
    }

    #[test]
    fn keeps_bracketed_host_with_port() {
        let entries = parse("[example.com]:2222 ssh-ed25519 AAAA");
        assert_eq!(entries[0].host, "[example.com]:2222");
    }

    #[test]
    fn ignores_comment_field_after_key() {
        let entries = parse("host ssh-ed25519 AAAA user@laptop");
        assert_eq!(entries[0].key_base64, "AAAA");
    }

    #[test]
    fn skips_malformed_lines_without_dropping_good_ones() {
        let entries = parse("brokenline\ngood ssh-ed25519 AAAA");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].host, "good");
    }

    #[test]
    fn lookup_key_matches_openssh_port_form() {
        assert_eq!(lookup_key("example.com", 22), "example.com");
        assert_eq!(lookup_key("example.com", 2222), "[example.com]:2222");
        assert_eq!(lookup_key("203.0.113.10", 22), "203.0.113.10");
    }

    #[test]
    fn a_port_22_pin_does_not_match_another_port() {
        // Entry stored for the default port must not verify a connection to a
        // different port of the same host.
        let entries = parse("example.com ssh-ed25519 AAAA");
        let key_for_2222 = lookup_key("example.com", 2222);
        assert!(!entries.iter().any(|e| e.host == key_for_2222));
    }
}
