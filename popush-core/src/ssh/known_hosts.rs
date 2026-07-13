use super::hostkey::KnownHost;

pub fn parse(contents: &str) -> Vec<KnownHost> {
    let mut out = Vec::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('@') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(hosts) = parts.next() else { continue };
        let Some(key_type) = parts.next() else {
            continue;
        };
        let Some(key_base64) = parts.next() else {
            continue;
        };
        if hosts.starts_with('|') {
            continue;
        }
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

pub fn lookup_key(host: &str, port: u16) -> String {
    if port == 22 {
        host.to_string()
    } else {
        format!("[{host}]:{port}")
    }
}

fn normalize_host(host: &str) -> String {
    if let Some(inner) = host.strip_prefix('[') {
        if let Some(closing) = inner.strip_suffix(']') {
            return closing.to_string();
        }
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
        let entries = parse("example.com ssh-ed25519 AAAA");
        let key_for_2222 = lookup_key("example.com", 2222);
        assert!(!entries.iter().any(|e| e.host == key_for_2222));
    }
}
