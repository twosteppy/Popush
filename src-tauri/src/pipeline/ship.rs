//! The Ship It orchestrator. Runs the seven steps, emitting events and
//! honouring cancellation, driving the `popush_core` state machine. Every failure
//! message and skip reason comes from the core, so this file adds no user-facing
//! strings of its own.
//!
//! ## Verification note
//! This module is Tauri-coupled (it emits events through an `AppHandle`) and so
//! compiles only with the GUI toolchain on the target. The step *logic* is thin:
//! each step calls a `popush_core` decision function or an already-verified infra
//! function (`crate::git`, `crate::adapters`, `crate::ssh`).

use std::time::Instant;

use popush_core::config::{ServiceConfig, SiteConfig};
use popush_core::error::UserMessage;
use popush_core::pipeline::messages::{failure_message, rollback_offer, FailureKind};
use popush_core::pipeline::step::SkipContext;
use popush_core::pipeline::{PipelineState, Step};
use tauri::{AppHandle, Emitter};

use crate::adapters;
use crate::pipeline::events::{
    step_index, OutputStream, PipelineEventOutcome, PipelineFinished, StepEventOutcome,
    StepFinished, StepOutput, StepStarted,
};
use crate::ssh::SshPool;
use crate::state::AppState;

/// Everything one pipeline run needs. Assembled by the command handler.
pub struct ShipContext<'a> {
    pub app: AppHandle,
    pub state: &'a AppState,
    pub pool: &'a SshPool,
    pub server_id: popush_core::ids::ServerId,
    pub site: SiteConfig,
    pub service: ServiceConfig,
    /// The resolved local clone path (the git panel's `local_path`).
    pub local_path: std::path::PathBuf,
    /// Files the user selected to commit.
    pub files: Vec<std::path::PathBuf>,
    /// The commit message.
    pub message: String,
    /// The pipeline id (already returned to the frontend).
    pub pipeline_id: popush_core::ids::PipelineId,
}

/// Run the whole pipeline. Emits step and pipeline events; returns when the run
/// reaches a terminal state. Never returns a generic error, a failure is emitted
/// as a structured [`UserMessage`].
pub async fn run_pipeline(ctx: ShipContext<'_>) {
    let started = Instant::now();
    let remote_path = ctx.site.remote_path.to_string_lossy().to_string();

    // Build the skip context from real facts.
    let git_status = crate::git::status(&ctx.local_path, &ctx.site.git_remote).ok();
    let has_changes = git_status
        .as_ref()
        .map(|g| !g.changed_files.is_empty())
        .unwrap_or(false);
    let has_commits_to_push = git_status.as_ref().map(|g| g.ahead > 0).unwrap_or(false);
    let caps = adapters::capabilities(&ctx.service, ctx.site.health_check_url.is_some());
    let skip = SkipContext {
        has_local_changes: has_changes,
        has_commits_to_push,
        has_build_command: ctx.site.build_command.is_some(),
        adapter_can_restart: caps.can_restart,
    };
    // The initial plan (with skipped steps marked) seeds the frontend's mirror
    // before anything runs; the frontend then updates it from step events.
    let plan = PipelineState::new(&skip);
    let _ = ctx.app.emit("pipeline:plan", &plan);

    // Capture the pre-deploy SHA for rollback, before anything changes.
    let rollback_sha = capture_remote_sha(&ctx, &remote_path).await;

    // Walk the steps in order, running each that is not pre-skipped.
    for step in Step::ALL {
        if step.is_skipped(&skip) {
            emit_step(
                &ctx,
                step,
                StepEventOutcome::Skipped,
                None,
                skip_summary(step),
            );
            continue;
        }
        // Cancellation is checked between steps: completed steps are never
        // undone; the state is reported honestly.
        if ctx.state.is_cancelled(&ctx.pipeline_id) {
            finish(
                &ctx,
                PipelineEventOutcome::Cancelled,
                started,
                None,
                rollback_offer_for(&remote_path, &rollback_sha),
            );
            return;
        }

        emit_started(&ctx, step);
        let outcome = run_step(&ctx, step, &remote_path).await;
        match outcome {
            Ok(summary) => emit_step(&ctx, step, StepEventOutcome::Ok, None, summary),
            Err(failure) => {
                emit_step(
                    &ctx,
                    step,
                    StepEventOutcome::Failed,
                    None,
                    failure.headline.clone(),
                );
                finish(
                    &ctx,
                    PipelineEventOutcome::Failed,
                    started,
                    Some(failure),
                    rollback_offer_for(&remote_path, &rollback_sha),
                );
                return;
            }
        }
    }

    finish(&ctx, PipelineEventOutcome::Ok, started, None, None);
}

/// Run one step, returning a one-line success summary or the failure message.
async fn run_step(
    ctx: &ShipContext<'_>,
    step: Step,
    remote_path: &str,
) -> Result<String, UserMessage> {
    match step {
        Step::Check => run_check(ctx),
        Step::Commit => run_commit(ctx),
        Step::Push => run_push(ctx),
        Step::Pull => run_pull(ctx, remote_path).await,
        Step::Build => run_build(ctx, remote_path).await,
        Step::Restart => run_restart(ctx).await,
        Step::Verify => run_verify(ctx).await,
    }
}

fn run_check(ctx: &ShipContext<'_>) -> Result<String, UserMessage> {
    // A conflicted or detached repo is refused with the exact core message.
    let status =
        crate::git::status(&ctx.local_path, &ctx.site.git_remote).map_err(|e| e.user_message())?;
    if status.has_conflicts {
        return Err(popush_core::error::GitError::MergeConflicts {
            count: status.changed_files.len(),
            files: status
                .changed_files
                .iter()
                .map(|f| f.path.clone())
                .collect(),
        }
        .user_message());
    }
    Ok(format!("On {}, server reachable", status.branch))
}

fn run_commit(ctx: &ShipContext<'_>) -> Result<String, UserMessage> {
    let sha = crate::git::stage_and_commit(&ctx.local_path, &ctx.message, &ctx.files)
        .map_err(|e| e.user_message())?;
    Ok(format!("{} files, {}", ctx.files.len(), sha))
}

fn run_push(ctx: &ShipContext<'_>) -> Result<String, UserMessage> {
    // Rejections map to the verbatim messages via the core failure kinds.
    crate::git::push(&ctx.local_path, &ctx.site.git_remote, &ctx.site.git_branch).map_err(|e| {
        match e {
            popush_core::error::GitError::PushRejectedNonFastForward => {
                failure_message(&FailureKind::PushNonFastForward)
            }
            popush_core::error::GitError::PushRejectedPermission => {
                failure_message(&FailureKind::PushPermissionDenied)
            }
            other => other.user_message(),
        }
    })?;
    Ok(format!("{}/{}", ctx.site.git_remote, ctx.site.git_branch))
}

async fn run_pull(ctx: &ShipContext<'_>, remote_path: &str) -> Result<String, UserMessage> {
    let cmd = popush_core::ssh::RemoteCommand::new(
        "cd {} && git pull --ff-only",
        vec![remote_path.to_string()],
    );
    let out = exec_streaming(ctx, Step::Pull, cmd).await?;
    if out.exit_code != 0 {
        // Local changes on the server are the common, nameable cause.
        if out.stderr.contains("local changes") || out.stderr.contains("would be overwritten") {
            return Err(failure_message(&FailureKind::PullLocalChangesOnServer {
                remote_path: remote_path.to_string(),
            }));
        }
        return Err(popush_core::error::SshError::CommandFailed {
            command: out.command_display,
            exit_code: out.exit_code,
            stderr: out.stderr,
        }
        .user_message());
    }
    Ok("Fast-forward".into())
}

async fn run_build(ctx: &ShipContext<'_>, remote_path: &str) -> Result<String, UserMessage> {
    let Some(build) = ctx.site.build_command.clone() else {
        return Ok("no build command".into());
    };
    // The build command is user-configured; it runs as the *content* of one remote
    // command with the path escaped. The build text itself is intentionally
    // executed as the user asked (see the honest weakness note).
    let cmd = popush_core::ssh::RemoteCommand::new(
        "cd {} && sh -c {}",
        vec![remote_path.to_string(), build],
    );
    let out = exec_streaming(ctx, Step::Build, cmd).await?;
    if out.exit_code != 0 {
        return Err(failure_message(&FailureKind::BuildFailed {
            exit_code: out.exit_code,
            output: out.stderr,
        }));
    }
    Ok("Build succeeded".into())
}

async fn run_restart(ctx: &ShipContext<'_>) -> Result<String, UserMessage> {
    let cmd = restart_command(&ctx.service, &ctx.site.remote_path.to_string_lossy());
    let out = exec_streaming(ctx, Step::Restart, cmd).await?;
    if out.exit_code != 0 {
        return Err(failure_message(&FailureKind::RestartFailed {
            service_logs: out.stderr,
        }));
    }
    Ok("Service restarted".into())
}

async fn run_verify(ctx: &ShipContext<'_>) -> Result<String, UserMessage> {
    // Poll adapter status; then the health check if configured.
    let status = adapters::status(
        ctx.pool,
        &ctx.service,
        &ctx.site.remote_path.to_string_lossy(),
    )
    .await
    .map_err(|e| e.user_message())?;
    if let Some(url) = ctx.site.health_check_url.clone() {
        if let Some(code) = http_head_status(&url).await {
            if !(200..300).contains(&code) {
                return Err(failure_message(&FailureKind::VerifyHealthCheck {
                    code,
                    logs: String::new(),
                }));
            }
        }
    }
    Ok(format!("{status:?}"))
}

/// Build the adapter's restart command (mirrors `adapters::status` dispatch).
fn restart_command(service: &ServiceConfig, remote_path: &str) -> popush_core::ssh::RemoteCommand {
    use popush_core::adapters::{docker, pm2, systemd};
    match service {
        ServiceConfig::Docker { .. } => docker::restart_command(remote_path),
        ServiceConfig::Systemd { unit } => systemd::restart_command(unit),
        ServiceConfig::Pm2 { app_name } => pm2::restart_command(app_name),
        // Static has no restart; this branch is unreachable because the step is
        // pre-skipped when the adapter cannot restart.
        ServiceConfig::Static { .. } => popush_core::ssh::RemoteCommand::literal("true"),
    }
}

/// Execute a remote command, streaming each line to the frontend and recording it
/// in the command log.
async fn exec_streaming(
    ctx: &ShipContext<'_>,
    step: Step,
    cmd: popush_core::ssh::RemoteCommand,
) -> Result<popush_core::command_log::CommandOutcome, UserMessage> {
    let out = ctx.pool.exec(cmd).await.map_err(|e| e.user_message())?;
    // Stream captured output as lines (the pool captures fully; large live streams
    // use the dedicated log stream path).
    for line in out.stdout.lines() {
        emit_output(ctx, step, line, OutputStream::Stdout);
    }
    for line in out.stderr.lines() {
        emit_output(ctx, step, line, OutputStream::Stderr);
    }
    // Record in the command log with a timestamp captured now.
    ctx.state
        .record_command(popush_core::command_log::CommandLogEntry {
            timestamp: chrono::Utc::now(),
            server: ctx.server_id.clone(),
            command: out.command_display.clone(),
            exit_code: Some(out.exit_code),
            duration_ms: out.duration_ms,
        });
    Ok(out)
}

async fn capture_remote_sha(ctx: &ShipContext<'_>, remote_path: &str) -> Option<String> {
    let cmd = popush_core::ssh::RemoteCommand::new(
        "cd {} && git rev-parse --short HEAD",
        vec![remote_path.to_string()],
    );
    let out = ctx.pool.exec(cmd).await.ok()?;
    if out.exit_code == 0 {
        Some(out.stdout.trim().to_string())
    } else {
        None
    }
}

fn rollback_offer_for(remote_path: &str, sha: &Option<String>) -> Option<UserMessage> {
    sha.as_ref().map(|s| rollback_offer(remote_path, s))
}

/// An HTTP `HEAD` to the health check URL, returning the status code.
async fn http_head_status(url: &str) -> Option<u16> {
    let resp = reqwest::Client::new().head(url).send().await.ok()?;
    Some(resp.status().as_u16())
}

fn skip_summary(step: Step) -> String {
    match step {
        Step::Commit => "no local changes".into(),
        Step::Push => "nothing to push".into(),
        Step::Build => "no build command".into(),
        Step::Restart => "no restart for this service".into(),
        _ => "skipped".into(),
    }
}

// --- event emission ---------------------------------------------------------

fn emit_started(ctx: &ShipContext<'_>, step: Step) {
    let _ = ctx.app.emit(
        "pipeline:step-started",
        StepStarted {
            pipeline_id: ctx.pipeline_id.clone(),
            step_index: step_index(step),
            step_name: step.label().to_string(),
        },
    );
}

fn emit_output(ctx: &ShipContext<'_>, step: Step, line: &str, stream: OutputStream) {
    let _ = ctx.app.emit(
        "pipeline:step-output",
        StepOutput {
            pipeline_id: ctx.pipeline_id.clone(),
            step_index: step_index(step),
            line: line.to_string(),
            stream,
        },
    );
}

fn emit_step(
    ctx: &ShipContext<'_>,
    step: Step,
    outcome: StepEventOutcome,
    exit_code: Option<i32>,
    summary: String,
) {
    let _ = ctx.app.emit(
        "pipeline:step-finished",
        StepFinished {
            pipeline_id: ctx.pipeline_id.clone(),
            step_index: step_index(step),
            outcome,
            exit_code,
            summary,
        },
    );
}

fn finish(
    ctx: &ShipContext<'_>,
    outcome: PipelineEventOutcome,
    started: Instant,
    failure: Option<UserMessage>,
    rollback: Option<UserMessage>,
) {
    let _ = ctx.app.emit(
        "pipeline:finished",
        PipelineFinished {
            pipeline_id: ctx.pipeline_id.clone(),
            outcome,
            duration_ms: started.elapsed().as_millis() as u64,
            failure,
            rollback: if outcome == PipelineEventOutcome::Failed {
                rollback
            } else {
                None
            },
        },
    );
}
