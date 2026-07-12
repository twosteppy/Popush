//! Service adapters (§9). Each adapter knows how to check, start, stop, restart,
//! and tail logs for one service type, and — crucially — what it *cannot* do,
//! via [`Capabilities`]. `capabilities()` is what stops the UI lying (§9.1): a
//! static site has no Restart button, rather than a Restart button that fails.
//!
//! The socket I/O (actually running the command) lives in the binary. Everything
//! here — the command each operation issues, and the parse from raw output to a
//! [`SiteStatus`] — is pure and golden-file tested against **real captured
//! output** (§23.1), because the parse is where honesty (D12) is won or lost.

pub mod docker;
pub mod pm2;
pub mod static_site;
pub mod systemd;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What an adapter can actually do. Drives which buttons render (§9.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct Capabilities {
    /// Whether start and stop are supported.
    pub can_start_stop: bool,
    /// Whether restart is supported.
    pub can_restart: bool,
    /// Whether logs can be tailed.
    pub has_logs: bool,
    /// Whether the reported status is trustworthy. False for static sites with no
    /// health check (§9.5): the UI shows amber Unknown rather than a fake green.
    pub status_is_reliable: bool,
}
