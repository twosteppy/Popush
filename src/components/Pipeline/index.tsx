import { useState } from 'react';
import {
  Check as CheckIcon,
  X,
  Loader2,
  Circle,
  Minus,
  Rocket,
  ExternalLink,
} from 'lucide-react';
import type {
  NextAction,
  Step,
  StepState,
  UserMessage,
} from '../../types/generated';
import { usePipelineStore, type PipelineStepView } from '../../store/pipeline';

const STEP_LABEL: Record<Step, string> = {
  check: 'Check',
  commit: 'Commit',
  push: 'Push',
  pull: 'Pull',
  build: 'Build',
  restart: 'Restart',
  verify: 'Verify',
};

/** A short, screen-reader-friendly description of the whole pipeline state. */
function announce(steps: PipelineStepView[], finished: boolean): string {
  const running = steps.find((s) => s.state.state === 'running');
  if (running) return `${STEP_LABEL[running.step]} in progress`;
  const failed = steps.find((s) => s.state.state === 'failed');
  if (failed) return `${STEP_LABEL[failed.step]} did not complete`;
  if (finished) return 'Pipeline finished';
  return '';
}

export function Pipeline({ liveUrl }: { liveUrl?: string | null }) {
  const steps = usePipelineStore((s) => s.steps);
  const finished = usePipelineStore((s) => s.finished);
  const outcome = usePipelineStore((s) => s.outcome);
  const failure = usePipelineStore((s) => s.failure);
  const rollback = usePipelineStore((s) => s.rollback);

  return (
    <div>
      {/* Announce step changes to assistive tech. */}
      <div aria-live="polite" className="sr-only">
        {announce(steps, finished)}
      </div>
      <ol className="flex flex-col gap-1">
        {steps.map((entry, index) => (
          <PipelineStep key={entry.step} entry={entry} index={index} />
        ))}
      </ol>

      {finished && outcome === 'ok' ? (
        <ShippedMessage liveUrl={liveUrl} />
      ) : null}
      {finished && failure ? <FailureMessage message={failure} /> : null}
      {finished && rollback ? <RollbackOffer message={rollback} /> : null}
    </div>
  );
}

/** The celebratory confirmation shown when every step passed and the site is
 * live. Mirrors the failure card, but in the success theme. */
function ShippedMessage({ liveUrl }: { liveUrl?: string | null }) {
  const safe =
    liveUrl && /^https?:\/\//.test(liveUrl) ? liveUrl : null;
  const host = safe ? safe.replace(/^https?:\/\//, '').replace(/\/$/, '') : null;
  return (
    <div
      role="status"
      className="mt-3 flex items-center gap-3 rounded-sm border-2 border-status-running/60 bg-status-running/10 p-3 shadow-hard-sm"
    >
      <span className="grid h-9 w-9 shrink-0 place-items-center rounded-full bg-status-running/15 text-status-running">
        <Rocket size={18} aria-hidden="true" />
      </span>
      <div className="min-w-0 flex-1">
        <p className="text-sm font-semibold text-status-running">
          Shipped and live
        </p>
        <p className="mt-0.5 text-sm text-text-secondary">
          Every step passed and the site is responding. Your changes are
          deployed.
        </p>
      </div>
      {safe && host ? (
        <a
          href={safe}
          target="_blank"
          rel="noreferrer"
          className="pressable label-mono flex shrink-0 items-center gap-1.5 rounded-sm border border-status-running/50 px-2.5 py-1.5 text-[11px] text-status-running shadow-hard-sm hover:bg-status-running/10"
        >
          Visit site
          <ExternalLink size={12} aria-hidden="true" />
        </a>
      ) : null}
    </div>
  );
}

function PipelineStep({
  entry,
  index,
}: {
  entry: PipelineStepView;
  index: number;
}) {
  const { step, state, output } = entry;
  const label = STEP_LABEL[step];
  const failed = state.state === 'failed';
  const running = state.state === 'running';

  return (
    <li
      className={`rounded-md px-2 py-1.5 transition-colors ${
        running ? 'bg-accent-muted/40' : 'hover:bg-surface-hover'
      }`}
    >
      <div className="flex items-center gap-2">
        <StepIcon state={state} />
        <span
          className={`text-sm ${
            state.state === 'pending'
              ? 'text-text-tertiary'
              : 'text-text-primary'
          }`}
        >
          {label}
        </span>
        {running ? (
          <span
            aria-hidden="true"
            className="relative ml-2 h-1 flex-1 overflow-hidden rounded-full bg-surface-hover"
          >
            <span className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-accent to-transparent motion-safe:animate-shimmer" />
          </span>
        ) : null}
        {state.state === 'ok' ? (
          <span className="ml-auto font-mono text-xs text-text-tertiary">
            {state.summary}
          </span>
        ) : null}
        {state.state === 'skipped' ? (
          <span className="ml-auto text-xs text-text-tertiary">skipped</span>
        ) : null}
      </div>

      {/* Failed steps auto-expand with the failure summary and captured output. */}
      {failed ? (
        <div className="mt-1 pl-6">
          <p className="font-mono text-xs text-status-failed">
            {state.summary}
          </p>
          {output.length > 0 ? (
            <pre
              data-testid={`step-output-${index}`}
              className="mt-1 max-h-40 overflow-auto whitespace-pre-wrap font-mono text-xs text-text-secondary"
            >
              {output.join('\n')}
            </pre>
          ) : null}
        </div>
      ) : null}
      {state.state === 'skipped' ? (
        <p className="mt-1 pl-6 text-xs text-text-tertiary">{state.reason}</p>
      ) : null}
    </li>
  );
}

function StepIcon({ state }: { state: StepState }) {
  const common = 'shrink-0';
  switch (state.state) {
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

/** The backend's specific failure explanation (never a generic message). */
function FailureMessage({ message }: { message: UserMessage }) {
  return (
    <div className="mt-3 rounded-sm border-2 border-status-failed/50 bg-surface-base p-3 shadow-hard-sm">
      <p className="text-sm font-semibold text-status-failed">
        {message.headline}
      </p>
      <p className="mt-1 text-sm text-text-secondary">{message.consequence}</p>
      <NextActionView action={message.next_action} />
    </div>
  );
}

function NextActionView({ action }: { action: NextAction }) {
  switch (action.kind) {
    case 'run_command':
      return <CopyableCommand command={action.command} />;
    case 'advice':
      return <p className="mt-2 text-sm text-text-secondary">{action.text}</p>;
    case 'open_flow':
      return <p className="mt-2 text-sm text-text-secondary">{action.label}</p>;
    case 'retry':
      return null;
  }
}

/** The rollback offer surfaced when a deploy failed mid-flight. */
function RollbackOffer({ message }: { message: UserMessage }) {
  return (
    <div className="mt-2 rounded-sm border border-border-strong bg-surface-raised p-3 shadow-hard-sm">
      <p className="text-sm font-medium text-text-primary">
        {message.headline}
      </p>
      <p className="mt-1 text-sm text-text-secondary">{message.consequence}</p>
      <NextActionView action={message.next_action} />
    </div>
  );
}

/** A monospaced command with a copy-to-clipboard affordance. */
function CopyableCommand({ command }: { command: string }) {
  const [copied, setCopied] = useState(false);

  async function copy() {
    try {
      await navigator.clipboard?.writeText(command);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    } catch {
      /* ignore */
    }
  }

  return (
    <div className="mt-2 flex items-center gap-2">
      <code className="min-w-0 flex-1 truncate rounded-sm border border-border-strong bg-surface-base px-2 py-1 font-mono text-xs text-text-primary">
        {command}
      </code>
      <button
        type="button"
        onClick={() => void copy()}
        className="pressable label-mono shrink-0 rounded-sm border border-border-strong px-2 py-1 text-[11px] text-text-secondary shadow-hard-sm hover:bg-surface-hover"
      >
        {copied ? 'Copied' : 'Copy'}
      </button>
    </div>
  );
}
