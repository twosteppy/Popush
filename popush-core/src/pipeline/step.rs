//! Pipeline steps and the state machine.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The seven steps of Ship It, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    /// Local repo clean of conflicts, on the right branch, server reachable.
    Check,
    /// Stage selected files, commit with the user's message.
    Commit,
    /// Push to the GitHub remote.
    Push,
    /// SSH to the VPS and `git pull` in the site's remote path.
    Pull,
    /// Run the site's build command in the remote path.
    Build,
    /// Adapter `restart`.
    Restart,
    /// Poll status and hit the health check.
    Verify,
}

impl Step {
    /// All steps in execution order.
    pub const ALL: [Step; 7] = [
        Step::Check,
        Step::Commit,
        Step::Push,
        Step::Pull,
        Step::Build,
        Step::Restart,
        Step::Verify,
    ];

    /// The label shown in the UI and used in messages.
    pub fn label(self) -> &'static str {
        match self {
            Step::Check => "Check",
            Step::Commit => "Commit",
            Step::Push => "Push",
            Step::Pull => "Pull",
            Step::Build => "Build",
            Step::Restart => "Restart",
            Step::Verify => "Verify",
        }
    }

    /// Whether this step mutates the server. Used by cancellation to warn
    /// when the site may be left in an inconsistent state.
    pub fn mutates_server(self) -> bool {
        matches!(self, Step::Pull | Step::Build | Step::Restart)
    }
}

/// The facts that decide which steps are skipped. Computed before the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkipContext {
    /// Whether there are local changes to commit.
    pub has_local_changes: bool,
    /// Whether there are commits to push.
    pub has_commits_to_push: bool,
    /// Whether the site has a build command.
    pub has_build_command: bool,
    /// Whether the adapter supports restart (false for static).
    pub adapter_can_restart: bool,
}

impl Step {
    /// Whether this step is skipped given the context. `Check`, `Pull`,
    /// and `Verify` are never skipped.
    pub fn is_skipped(self, ctx: &SkipContext) -> bool {
        match self {
            Step::Check | Step::Pull | Step::Verify => false,
            Step::Commit => !ctx.has_local_changes,
            Step::Push => !ctx.has_commits_to_push && !ctx.has_local_changes,
            Step::Build => !ctx.has_build_command,
            Step::Restart => !ctx.adapter_can_restart,
        }
    }
}

/// The live state of one step in the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum StepState {
    /// Not yet reached.
    Pending,
    /// Skipped, with the reason for the one-line summary.
    Skipped {
        /// Why the step was skipped.
        reason: String,
    },
    /// Currently running.
    Running,
    /// Finished successfully, with a one-line summary and duration.
    Ok {
        /// One-line result summary.
        summary: String,
        /// How long the step took, in milliseconds.
        duration_ms: u64,
    },
    /// Failed; the UI expands this and shows stderr.
    Failed {
        /// One-line failure summary.
        summary: String,
        /// How long the step ran before failing, in milliseconds.
        duration_ms: u64,
    },
}

/// The terminal outcome of a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum StepOutcome {
    /// The step succeeded.
    Ok,
    /// The step failed; the pipeline stops.
    Failed,
}

/// The whole pipeline's live state: each step and its state. The frontend mirrors
/// this from events; it never computes transitions itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PipelineState {
    /// Per-step state, indexed parallel to [`Step::ALL`].
    pub steps: Vec<StepEntry>,
    /// Whether the pipeline has finished (all done, failed, or cancelled).
    pub finished: bool,
    /// The pre-deploy git SHA, captured for rollback. `None` until Check.
    pub rollback_sha: Option<String>,
}

/// One step's identity plus its state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct StepEntry {
    /// Which step.
    pub step: Step,
    /// Its current state.
    pub state: StepState,
}

impl PipelineState {
    /// Build the initial state, marking skipped steps up front so the UI shows the
    /// real plan before anything runs.
    pub fn new(ctx: &SkipContext) -> Self {
        let steps = Step::ALL
            .iter()
            .map(|&step| StepEntry {
                step,
                state: if step.is_skipped(ctx) {
                    StepState::Skipped {
                        reason: skip_reason(step),
                    }
                } else {
                    StepState::Pending
                },
            })
            .collect();
        Self {
            steps,
            finished: false,
            rollback_sha: None,
        }
    }

    /// The next step that should run (first non-skipped, non-finished). `None`
    /// when the pipeline is complete.
    pub fn next_runnable(&self) -> Option<Step> {
        self.steps
            .iter()
            .find(|e| matches!(e.state, StepState::Pending))
            .map(|e| e.step)
    }
}

fn skip_reason(step: Step) -> String {
    match step {
        Step::Commit => "no local changes".into(),
        Step::Push => "nothing to push".into(),
        Step::Build => "no build command configured".into(),
        Step::Restart => "this service type has no restart".into(),
        _ => "skipped".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_ctx() -> SkipContext {
        SkipContext {
            has_local_changes: true,
            has_commits_to_push: true,
            has_build_command: true,
            adapter_can_restart: true,
        }
    }

    #[test]
    fn full_pipeline_skips_nothing() {
        let state = PipelineState::new(&full_ctx());
        assert!(state
            .steps
            .iter()
            .all(|e| !matches!(e.state, StepState::Skipped { .. })));
        assert_eq!(state.next_runnable(), Some(Step::Check));
    }

    #[test]
    fn static_site_with_no_changes_skips_commit_push_build_restart() {
        let ctx = SkipContext {
            has_local_changes: false,
            has_commits_to_push: false,
            has_build_command: false,
            adapter_can_restart: false,
        };
        let state = PipelineState::new(&ctx);
        let skipped: Vec<Step> = state
            .steps
            .iter()
            .filter(|e| matches!(e.state, StepState::Skipped { .. }))
            .map(|e| e.step)
            .collect();
        assert_eq!(
            skipped,
            vec![Step::Commit, Step::Push, Step::Build, Step::Restart]
        );
        // Check, Pull, Verify always run.
        assert_eq!(state.next_runnable(), Some(Step::Check));
    }

    #[test]
    fn check_pull_verify_are_never_skipped() {
        let ctx = SkipContext {
            has_local_changes: false,
            has_commits_to_push: false,
            has_build_command: false,
            adapter_can_restart: false,
        };
        for step in [Step::Check, Step::Pull, Step::Verify] {
            assert!(!step.is_skipped(&ctx), "{step:?} must never be skipped");
        }
    }

    #[test]
    fn push_runs_when_there_are_local_changes_even_if_nothing_queued() {
        // After Commit produces a new commit, Push must not be pre-skipped.
        let ctx = SkipContext {
            has_local_changes: true,
            has_commits_to_push: false,
            has_build_command: false,
            adapter_can_restart: false,
        };
        assert!(!Step::Push.is_skipped(&ctx));
    }

    #[test]
    fn mutating_steps_are_flagged_for_cancellation_warnings() {
        assert!(Step::Build.mutates_server());
        assert!(Step::Restart.mutates_server());
        assert!(Step::Pull.mutates_server());
        assert!(!Step::Check.mutates_server());
        assert!(!Step::Push.mutates_server());
    }
}
