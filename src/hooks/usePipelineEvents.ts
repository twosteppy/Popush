// Subscribes to the backend pipeline event stream and mirrors it into the
// pipeline store. Outside Tauri (dev/test) `listen` no-ops, so nothing here
// runs.

import { useEffect } from 'react';
import { listen } from '../lib/ipc';
import { usePipelineStore } from '../store/pipeline';
import type { PipelineState, StepState, UserMessage } from '../types/generated';

/** `pipeline:step-started` payload. */
interface StepStartedEvent {
  pipeline_id: string;
  step_index: number;
  step_name: string;
}

/** `pipeline:step-output` payload. */
interface StepOutputEvent {
  pipeline_id: string;
  step_index: number;
  line: string;
  stream: 'stdout' | 'stderr';
}

/** `pipeline:step-finished` payload. */
interface StepFinishedEvent {
  pipeline_id: string;
  step_index: number;
  outcome: 'ok' | 'failed' | 'skipped';
  exit_code: number | null;
  summary: string;
}

/** `pipeline:finished` payload. */
interface PipelineFinishedEvent {
  pipeline_id: string;
  outcome: 'ok' | 'failed' | 'cancelled';
  duration_ms: number;
  failure: UserMessage | null;
  rollback: UserMessage | null;
}

/** Build the terminal StepState from a step-finished event payload. */
function terminalState(event: StepFinishedEvent): StepState {
  switch (event.outcome) {
    case 'ok':
      return { state: 'ok', summary: event.summary, duration_ms: 0n };
    case 'failed':
      return { state: 'failed', summary: event.summary, duration_ms: 0n };
    case 'skipped':
      return { state: 'skipped', reason: event.summary };
  }
}

/**
 * Subscribe to the backend pipeline event stream for the lifetime of the
 * component. All handlers are pure store mutations.
 */
export function usePipelineEvents(): void {
  const { plan, stepStarted, appendOutput, stepFinished, finish } =
    usePipelineStore();

  useEffect(() => {
    const unsubs: Array<() => void> = [];
    let cancelled = false;

    function track(promise: Promise<() => void>) {
      void promise.then((unlisten) => {
        if (cancelled) unlisten();
        else unsubs.push(unlisten);
      });
    }

    track(
      listen<PipelineState>('pipeline:plan', (state) => {
        plan(state);
      }),
    );
    track(
      listen<StepStartedEvent>('pipeline:step-started', (e) => {
        stepStarted(e.step_index);
      }),
    );
    track(
      listen<StepOutputEvent>('pipeline:step-output', (e) => {
        appendOutput(e.step_index, e.line);
      }),
    );
    track(
      listen<StepFinishedEvent>('pipeline:step-finished', (e) => {
        stepFinished(e.step_index, terminalState(e));
      }),
    );
    track(
      listen<PipelineFinishedEvent>('pipeline:finished', (e) => {
        finish(e.outcome, e.failure, e.rollback);
      }),
    );

    return () => {
      cancelled = true;
      for (const unlisten of unsubs) unlisten();
    };
  }, [plan, stepStarted, appendOutput, stepFinished, finish]);
}
