//! SSH subsystem (§8). The pure, headless-testable parts live here: command
//! construction ([`command`]) and the host-key verification *decision* logic
//! ([`hostkey`]). The socket-level I/O, `russh` sessions, the connection pool,
//! `ssh-agent` delegation, lives in the `src-tauri` binary, which links the
//! native libraries and runs on the Fedora target (see `docs/DECISIONS.md`).

pub mod command;
pub mod hostkey;
pub mod known_hosts;

pub use command::RemoteCommand;
pub use hostkey::{HostKeyDecision, HostKeyVerifier, KnownHost};
pub use known_hosts::parse as parse_known_hosts;
