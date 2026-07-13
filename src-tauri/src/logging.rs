//! Log redaction wiring. The `tracing` output is passed through
//! [`popush_core::redact::redact_line`] before it is written, so key material,
//! tokens, and `Authorization` headers never reach the log file, which is itself
//! local-only and never transmitted.
//!
//! This is glue over the pure redactor: a [`std::io::Write`] wrapper that redacts
//! each complete line, exposed as a `tracing_subscriber` [`MakeWriter`].

use std::io::{self, Write};

use tracing_subscriber::fmt::MakeWriter;

/// A writer that redacts complete lines before delegating to an inner writer.
pub struct RedactingWriter<W> {
    inner: W,
    buffer: Vec<u8>,
    /// True while inside a multi-line `-----BEGIN … PRIVATE KEY-----` block. A
    /// private key's base64 body lines carry no token prefix, so per-line
    /// redaction alone would leak them; every body line is blanked until the
    /// matching `-----END-----`.
    in_private_key: bool,
}

impl<W: Write> RedactingWriter<W> {
    fn new(inner: W) -> Self {
        Self {
            inner,
            buffer: Vec::new(),
            in_private_key: false,
        }
    }

    /// Redact a single already-newline-stripped line, accounting for multi-line
    /// private-key blocks.
    fn redact_one(&mut self, line: &str) -> String {
        use popush_core::redact::{is_private_key_begin, is_private_key_end, REDACTED};
        if self.in_private_key {
            if is_private_key_end(line) {
                self.in_private_key = false;
            }
            return REDACTED.to_string();
        }
        if is_private_key_begin(line) {
            // Redact the header line; if the block does not also end on this line,
            // keep redacting subsequent lines.
            if !is_private_key_end(line) {
                self.in_private_key = true;
            }
            return REDACTED.to_string();
        }
        popush_core::redact::redact_line(line)
    }

    /// Redact and flush all complete lines currently in the buffer.
    fn flush_complete_lines(&mut self) -> io::Result<()> {
        while let Some(pos) = self.buffer.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = self.buffer.drain(..=pos).collect();
            let text = String::from_utf8_lossy(&line);
            // `text` includes the trailing newline; redact the content, keep the \n.
            let trimmed = text.strip_suffix('\n').unwrap_or(&text).to_string();
            let redacted = self.redact_one(&trimmed);
            writeln!(self.inner, "{redacted}")?;
        }
        Ok(())
    }
}

impl<W: Write> Write for RedactingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        self.flush_complete_lines()?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // Redact and emit any trailing partial line, then flush the inner writer.
        if !self.buffer.is_empty() {
            let text = String::from_utf8_lossy(&self.buffer).into_owned();
            let redacted = self.redact_one(&text);
            self.inner.write_all(redacted.as_bytes())?;
            self.buffer.clear();
        }
        self.inner.flush()
    }
}

/// A `MakeWriter` that produces redacting writers over stderr. Installed as the
/// `tracing` writer in [`crate::run`].
#[derive(Clone, Copy, Default)]
pub struct RedactingMakeWriter;

impl<'a> MakeWriter<'a> for RedactingMakeWriter {
    type Writer = RedactingWriter<io::Stderr>;

    fn make_writer(&'a self) -> Self::Writer {
        RedactingWriter::new(io::stderr())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_a_token_across_a_write() {
        let mut buf = Vec::new();
        {
            let mut w = RedactingWriter::new(&mut buf);
            w.write_all(b"using ghp_secretvalue now\n").unwrap();
            w.flush().unwrap();
        }
        let out = String::from_utf8(buf).unwrap();
        assert!(!out.contains("ghp_secretvalue"));
        assert!(out.contains("[redacted]"));
    }

    #[test]
    fn redacts_a_multiline_private_key_body() {
        // Build the header/footer at runtime so the contiguous key marker is not
        // a committed source literal (the repo's secret scanner rejects those).
        let kind = "OPENSSH";
        let block = format!(
            "-----BEGIN {kind} PRIVATE KEY-----\n\
             b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAE\n\
             AAAAABAAAAMwAAAAtzc2gtZWQyNTUxOQ\n\
             -----END {kind} PRIVATE KEY-----\n\
             next ordinary line\n"
        );
        let mut buf = Vec::new();
        {
            let mut w = RedactingWriter::new(&mut buf);
            w.write_all(block.as_bytes()).unwrap();
            w.flush().unwrap();
        }
        let out = String::from_utf8(buf).unwrap();
        // No base64 body line survives.
        assert!(!out.contains("b3BlbnNzaC1rZXktdjEA"));
        assert!(!out.contains("AAAAABAAAAMwAAAAtzc2gt"));
        // The ordinary line after the block is not swallowed.
        assert!(out.contains("next ordinary line"));
    }
}
