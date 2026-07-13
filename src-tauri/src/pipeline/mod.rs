//! Ship It pipeline orchestration. The *state machine, skip logic, and
//! messages* are `popush_core::pipeline`; this layer runs the steps over SSH and
//! git, emits the typed events, captures the rollback SHA, and
//! honours cancellation. It is glue: every decision and every string
//! comes from the core.

pub mod events;
pub mod ship;

pub use ship::run_pipeline;
