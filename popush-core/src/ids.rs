//! Newtype identifiers. Distinct types stop a `SiteId` being passed where a
//! `ServerId` is expected — a class of bug a bare `String` would allow.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Identifies a server (a VPS) in the config.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct ServerId(pub String);

/// Identifies a site (one service on a server) in the config.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct SiteId(pub String);

/// Identifies one in-flight pipeline run. A random v4 UUID so two Ship It runs
/// never collide, even for the same site.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct PipelineId(pub String);

/// Identifies one live log stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct StreamId(pub String);

impl ServerId {
    /// Borrow the underlying identifier string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl SiteId {
    /// Borrow the underlying identifier string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PipelineId {
    /// Create a fresh, unique pipeline id.
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for PipelineId {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamId {
    /// Create a fresh, unique stream id.
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for StreamId {
    fn default() -> Self {
        Self::new()
    }
}
