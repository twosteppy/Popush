//! Ship It pipeline orchestration (§12). The *state machine, skip logic, and
//! messages* are `popush_core::pipeline`; this layer runs the steps over SSH and
//! git, emits the typed events (§6.2), captures the rollback SHA (§12.5), and
//! honours cancellation (§12.6). It is glue (D14): every decision and every string
//! comes from the core.

pub mod events;
pub mod ship;

pub use ship::run_pipeline;
