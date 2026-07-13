pub mod docker;
pub mod pm2;
pub mod static_site;
pub mod systemd;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct Capabilities {
    pub can_start_stop: bool,
    pub can_restart: bool,
    pub has_logs: bool,
    pub status_is_reliable: bool,
}
