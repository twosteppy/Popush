use std::io::{self, Write};

use tracing_subscriber::fmt::MakeWriter;

pub struct RedactingWriter<W> {
    inner: W,
    buffer: Vec<u8>,
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

    fn redact_one(&mut self, line: &str) -> String {
        use popush_core::redact::{is_private_key_begin, is_private_key_end, REDACTED};
        if self.in_private_key {
            if is_private_key_end(line) {
                self.in_private_key = false;
            }
            return REDACTED.to_string();
        }
        if is_private_key_begin(line) {
            if !is_private_key_end(line) {
                self.in_private_key = true;
            }
            return REDACTED.to_string();
        }
        popush_core::redact::redact_line(line)
    }

    fn flush_complete_lines(&mut self) -> io::Result<()> {
        while let Some(pos) = self.buffer.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = self.buffer.drain(..=pos).collect();
            let text = String::from_utf8_lossy(&line);
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
        if !self.buffer.is_empty() {
            let text = String::from_utf8_lossy(&self.buffer).into_owned();
            let redacted = self.redact_one(&text);
            self.inner.write_all(redacted.as_bytes())?;
            self.buffer.clear();
        }
        self.inner.flush()
    }
}

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
        assert!(!out.contains("b3BlbnNzaC1rZXktdjEA"));
        assert!(!out.contains("AAAAABAAAAMwAAAAtzc2gt"));
        assert!(out.contains("next ordinary line"));
    }
}
