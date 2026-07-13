use popush_core::error::UserMessage;
use popush_core::ids::PipelineId;
use popush_core::pipeline::Step;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StepStarted {
    pub pipeline_id: PipelineId,
    pub step_index: usize,
    pub step_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepOutput {
    pub pipeline_id: PipelineId,
    pub step_index: usize,
    pub line: String,
    pub stream: OutputStream,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepFinished {
    pub pipeline_id: PipelineId,
    pub step_index: usize,
    pub outcome: StepEventOutcome,
    pub exit_code: Option<i32>,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StepEventOutcome {
    Ok,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineFinished {
    pub pipeline_id: PipelineId,
    pub outcome: PipelineEventOutcome,
    pub duration_ms: u64,
    pub failure: Option<UserMessage>,
    pub rollback: Option<UserMessage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineEventOutcome {
    Ok,
    Failed,
    Cancelled,
}

pub fn step_index(step: Step) -> usize {
    Step::ALL.iter().position(|&s| s == step).unwrap_or(0)
}
