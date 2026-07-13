//! # popush-core
//!
//! All of Popush's business logic, GUI-free and headless-testable. Built by
//! twostep. See the crate `README.md` and `docs/DECISIONS.md`.
//!
//! This crate deliberately links no windowing, webview, or native-SSH libraries.
//! It holds the parts of Popush that must be *correct* and are therefore worth
//! testing in isolation and in CI without a display:
//!
//! * [`ssh::command`], [`ssh::RemoteCommand`], the choke point through which
//!   every remote command passes, with mandatory shell escaping and the
//!   adversarial corpus. **The most security-critical code in Popush.**
//! * [`error`], the structured error taxonomy; every variant produces a
//!   [`error::UserMessage`] answering what happened, what it means, and what to do.
//! * [`config`], load, validate, and migrate `config.toml`, rejecting malformed
//!   config with a message that names the field.
//! * [`adapters`], parsing real service output into an honest
//!   [`config::SiteStatus`], golden-file tested.
//! * [`git::remote`], HTTPS/SSH remote classification and conversion.
//! * [`pipeline`], the Ship It state machine and its verbatim failure messages,
//!   with the banned-strings guarantee.
//! * [`wizard`], preview-then-apply fixes, including the by-construction
//!   guarantee that a key is never overwritten.
//!
//! The Tauri binary in `src-tauri` wires these into IPC commands and events; it
//! adds the socket I/O (`russh`, `git2`, `keyring`, `notify`) but no logic.

#![deny(missing_docs)]

pub mod adapters;
pub mod command_log;
pub mod config;
pub mod error;
pub mod git;
pub mod github;
pub mod ids;
pub mod pipeline;
pub mod redact;
pub mod ssh;
pub mod wizard;

/// The crate version, surfaced in the About dialog alongside the twostep credit.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The author credit that must survive any refactor.
pub const AUTHOR: &str = "twostep";
