//! The Ship It pipeline (§12): the seven-step chain from "I changed a file" to
//! "the change is live", with skip logic, live step state, and — above all —
//! honest failure messages that name the failing step and never say "Deploy
//! failed" (D11). The state machine and messages are pure and tested here; the
//! execution (running each step over SSH/git, emitting events, cancellation) is
//! orchestrated in the binary from these pieces.

pub mod messages;
pub mod step;

pub use messages::{failure_message, FailureKind};
pub use step::{PipelineState, Step, StepOutcome, StepState};
