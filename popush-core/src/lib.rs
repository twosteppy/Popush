//! # popush-core
//!
//! All of Popush's business logic, GUI-free and headless-testable. Built by
//! twostep. See the crate `README.md` and `docs/DECISIONS.md`.
//!
//! This crate deliberately links no windowing, webview, or native-SSH libraries.
//! It holds the parts of Popush that must be *correct* and are therefore worth
//! testing in isolation and in CI without a display:
//!
//! * [`ssh::command`] — [`ssh::RemoteCommand`], the choke point through which
//!   every remote command passes, with mandatory shell escaping (D10) and the
//!   adversarial corpus (§23.2). **The most security-critical code in Popush.**
//! * [`error`] — the structured error taxonomy; every variant produces a
//!   [`error::UserMessage`] answering the three questions of §16.1 (D11).
//! * [`config`] — load, validate, and migrate `config.toml`, rejecting malformed
//!   config with a message that names the field (§7).
//! * [`adapters`] — parsing real service output into an honest [`config::SiteStatus`]
//!   (D12), golden-file tested.
//! * [`git::remote`] — HTTPS/SSH remote classification and conversion (§10.3).
//! * [`pipeline`] — the Ship It state machine and its verbatim failure messages
//!   (§12), with the banned-strings guarantee (D11).
//! * [`wizard`] — preview-then-apply fixes, including the by-construction
//!   guarantee that a key is never overwritten (D13).
//!
//! The Tauri binary in `src-tauri` wires these into IPC commands and events; it
//! adds the socket I/O (`russh`, `git2`, `keyring`, `notify`) but no logic (D14).

#![deny(missing_docs)]

pub mod adapters;
pub mod command_log;
pub mod config;
pub mod error;
pub mod git;
pub mod ids;
pub mod pipeline;
pub mod ssh;
pub mod wizard;

/// The crate version, surfaced in the About dialog alongside the twostep credit.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The author credit that must survive any refactor (D9).
pub const AUTHOR: &str = "twostep";
