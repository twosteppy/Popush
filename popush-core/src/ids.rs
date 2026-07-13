use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct ServerId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct SiteId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct PipelineId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct StreamId(pub String);

impl ServerId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl SiteId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PipelineId {
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
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for StreamId {
    fn default() -> Self {
        Self::new()
    }
}
