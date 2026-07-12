//! Typed pipeline event payloads emitted to the frontend (§6.2). Names match the
//! IPC contract exactly; the frontend mirrors pipeline state from these (§6.3).

use popush_core::error::UserMessage;
use popush_core::ids::PipelineId;
use popush_core::pipeline::Step;
use serde::Serialize;

/// `pipeline:step-started`.
#[derive(Debug, Clone, Serialize)]
pub struct StepStarted {
    pub pipeline_id: PipelineId,
    pub step_index: usize,
    pub step_name: String,
}

/// `pipeline:step-output`, one line of live output from a running step.
#[derive(Debug, Clone, Serialize)]
pub struct StepOutput {
    pub pipeline_id: PipelineId,
    pub step_index: usize,
    pub line: String,
    pub stream: OutputStream,
}

/// Which stream a line of output came from.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputStream {
    Stdout,
    Stderr,
}

/// `pipeline:step-finished`.
#[derive(Debug, Clone, Serialize)]
pub struct StepFinished {
    pub pipeline_id: PipelineId,
    pub step_index: usize,
    pub outcome: StepEventOutcome,
    pub exit_code: Option<i32>,
    /// A one-line summary for the collapsed row (§12.2), or the failure message.
    pub summary: String,
}

/// The outcome carried by a step-finished event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StepEventOutcome {
    Ok,
    Failed,
    Skipped,
}

/// `pipeline:finished`.
#[derive(Debug, Clone, Serialize)]
pub struct PipelineFinished {
    pub pipeline_id: PipelineId,
    pub outcome: PipelineEventOutcome,
    pub duration_ms: u64,
    /// On failure, the full user-facing message (§12.4), never a generic string.
    pub failure: Option<UserMessage>,
    /// On failure, the rollback offer (§12.5): previous SHA + previewed command.
    pub rollback: Option<UserMessage>,
}

/// The terminal outcome of the whole pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineEventOutcome {
    Ok,
    Failed,
    Cancelled,
}

/// The step index of a [`Step`] within the fixed seven-step order, for events.
pub fn step_index(step: Step) -> usize {
    Step::ALL.iter().position(|&s| s == step).unwrap_or(0)
}
