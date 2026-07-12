// Pipeline — the seven-step vertical list, rendered entirely from PipelineState
// (D14: the frontend does not know the steps' meaning; it renders the enum).
//
//   pending  ○
//   running  spinner
//   ok       ✓ with duration
//   failed   ✗ (auto-expanded)
//   skipped  – with reason
//
// §20 / Phase 8 gate: step changes are announced via an aria-live region.

import { Check as CheckIcon, X, Loader2, Circle, Minus } from 'lucide-react';
import type { PipelineState, Step, StepEntry } from '../../types/generated';

interface PipelineProps {
  state: PipelineState | null;
}

const STEP_LABEL: Record<Step, string> = {
  check: 'Check',
  commit: 'Commit',
  push: 'Push',
  pull: 'Pull',
  build: 'Build',
  restart: 'Restart',
  verify: 'Verify',
};

function formatDuration(ms: bigint): string {
  const seconds = Number(ms) / 1000;
  if (seconds < 1) return `${Number(ms)}ms`;
  return `${seconds.toFixed(1)}s`;
}

/** A short, screen-reader-friendly description of the whole pipeline state. */
function announce(state: PipelineState | null): string {
  if (!state) return '';
  const running = state.steps.find((s) => s.state.state === 'running');
  if (running) return `${STEP_LABEL[running.step]} in progress`;
  const failed = state.steps.find((s) => s.state.state === 'failed');
  if (failed) return `${STEP_LABEL[failed.step]} did not complete`;
  if (state.finished) return 'Pipeline finished';
  return '';
}

export function Pipeline({ state }: PipelineProps) {
  const steps = state?.steps ?? [];

  return (
    <div>
      {/* §20: announce step changes to assistive tech. */}
      <div aria-live="polite" className="sr-only">
        {announce(state)}
      </div>
      <ol className="flex flex-col gap-1">
        {steps.map((entry) => (
          <PipelineStep key={entry.step} entry={entry} />
        ))}
      </ol>
    </div>
  );
}

function PipelineStep({ entry }: { entry: StepEntry }) {
  const { step, state } = entry;
  const label = STEP_LABEL[step];
  const failed = state.state === 'failed';

  return (
    <li className="rounded-md px-2 py-1.5">
      <div className="flex items-center gap-2">
        <StepIcon entry={entry} />
        <span
          className={`text-sm ${
            state.state === 'pending'
              ? 'text-text-tertiary'
              : 'text-text-primary'
          }`}
        >
          {label}
        </span>
        {state.state === 'ok' ? (
          <span className="ml-auto font-mono text-xs text-text-tertiary">
            {formatDuration(state.duration_ms)}
          </span>
        ) : null}
        {state.state === 'skipped' ? (
          <span className="ml-auto text-xs text-text-tertiary">skipped</span>
        ) : null}
      </div>

      {/* Failed steps auto-expand with the failure summary. */}
      {failed ? (
        <p className="mt-1 pl-6 font-mono text-xs text-status-failed">
          {state.summary}
        </p>
      ) : null}
      {state.state === 'skipped' ? (
        <p className="mt-1 pl-6 text-xs text-text-tertiary">{state.reason}</p>
      ) : null}
    </li>
  );
}

function StepIcon({ entry }: { entry: StepEntry }) {
  const s = entry.state.state;
  const common = 'shrink-0';
  switch (s) {
    case 'pending':
      return (
        <Circle
          size={14}
          className={`${common} text-text-tertiary`}
          aria-label="Pending"
        />
      );
    case 'running':
      return (
        <Loader2
          size={14}
          className={`${common} text-status-working motion-safe:animate-spin`}
          aria-label="Running"
        />
      );
    case 'ok':
      return (
        <CheckIcon
          size={14}
          className={`${common} text-status-running`}
          aria-label="Done"
        />
      );
    case 'failed':
      return (
        <X
          size={14}
          className={`${common} text-status-failed`}
          aria-label="Did not complete"
        />
      );
    case 'skipped':
      return (
        <Minus
          size={14}
          className={`${common} text-text-tertiary`}
          aria-label="Skipped"
        />
      );
  }
}
