//! The setup wizard (§11): the flow that removes the terminal. Every fix is
//! preview-then-apply, never destructive, always reversible (D13).
//!
//! The pure decision logic lives here, which check to run, what a fix would do,
//! and the by-construction guarantee that key generation can never overwrite an
//! existing key. The I/O (running `ssh -T git@github.com`, reading server key
//! files over SSH, clipboard, opening URLs) lives in the binary.

pub mod checks;
pub mod fixes;

pub use checks::{Check, CheckStatus};
pub use fixes::{key_generation_fix, remote_conversion_fix, Fix, FixPreview};
