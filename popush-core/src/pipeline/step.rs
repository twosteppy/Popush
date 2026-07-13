use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    Check,
    Commit,
    Push,
    Pull,
    Build,
    Restart,
    Verify,
}

impl Step {
    pub const ALL: [Step; 7] = [
        Step::Check,
        Step::Commit,
        Step::Push,
        Step::Pull,
        Step::Build,
        Step::Restart,
        Step::Verify,
    ];

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

    pub fn mutates_server(self) -> bool {
        matches!(self, Step::Pull | Step::Build | Step::Restart)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkipContext {
    pub has_local_changes: bool,
    pub has_commits_to_push: bool,
    pub has_build_command: bool,
    pub adapter_can_restart: bool,
}

impl Step {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum StepState {
    Pending,
    Skipped { reason: String },
    Running,
    Ok { summary: String, duration_ms: u64 },
    Failed { summary: String, duration_ms: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum StepOutcome {
    Ok,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PipelineState {
    pub steps: Vec<StepEntry>,
    pub finished: bool,
    pub rollback_sha: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct StepEntry {
    pub step: Step,
    pub state: StepState,
}

impl PipelineState {
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
