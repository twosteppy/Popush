import { create } from 'zustand';
import type {
  PipelineState,
  Step,
  StepState,
  UserMessage,
} from '../types/generated';

/** How the pipeline as a whole concluded. */
export type PipelineOutcome = 'ok' | 'failed' | 'cancelled';

/** A mirrored step plus the output lines the backend streamed for it. */
export interface PipelineStepView {
  step: Step;
  state: StepState;
  /** Streamed output lines for this step (stdout/stderr interleaved). */
  output: string[];
}

interface PipelineStore {
  /** The id of the pipeline currently being mirrored, if any. */
  pipelineId: string | null;
  /** Per-step mirror, seeded from the `pipeline:plan` event. */
  steps: PipelineStepView[];
  /** Whether the pipeline has concluded. */
  finished: boolean;
  /** How it concluded, once finished. */
  outcome: PipelineOutcome | null;
  /** The failure message to display, if it failed. */
  failure: UserMessage | null;
  /** The rollback offer to display, if one was made. */
  rollback: UserMessage | null;
  /** True while the drawer/log height is being remembered. */
  drawerHeight: number;
  drawerOpen: boolean;

  /** Begin mirroring a fresh pipeline; clears prior state. */
  begin: (pipelineId: string) => void;
  /** Seed the steps from an authoritative plan snapshot. */
  plan: (state: PipelineState) => void;
  /** Mark the step at `index` as running. */
  stepStarted: (index: number) => void;
  /** Append a streamed output line to the step at `index`. */
  appendOutput: (index: number, line: string) => void;
  /** Apply the terminal state for the step at `index`. */
  stepFinished: (index: number, state: StepState) => void;
  /** Record the pipeline conclusion and any failure/rollback messages. */
  finish: (
    outcome: PipelineOutcome,
    failure: UserMessage | null,
    rollback: UserMessage | null,
  ) => void;
  reset: () => void;
  setDrawerHeight: (height: number) => void;
  toggleDrawer: () => void;
  setDrawerOpen: (open: boolean) => void;
}

const EMPTY = {
  pipelineId: null,
  steps: [] as PipelineStepView[],
  finished: false,
  outcome: null,
  failure: null,
  rollback: null,
};

export const usePipelineStore = create<PipelineStore>((set) => ({
  ...EMPTY,
  drawerHeight: 220,
  drawerOpen: false,

  begin: (pipelineId) => set({ ...EMPTY, pipelineId }),

  plan: (state) =>
    set({
      steps: state.steps.map((entry) => ({
        step: entry.step,
        state: entry.state,
        output: [],
      })),
      finished: state.finished,
    }),

  stepStarted: (index) =>
    set((s) => ({
      steps: s.steps.map((step, i) =>
        i === index ? { ...step, state: { state: 'running' } } : step,
      ),
    })),

  appendOutput: (index, line) =>
    set((s) => ({
      steps: s.steps.map((step, i) =>
        i === index ? { ...step, output: [...step.output, line] } : step,
      ),
    })),

  stepFinished: (index, state) =>
    set((s) => ({
      steps: s.steps.map((step, i) =>
        i === index ? { ...step, state } : step,
      ),
    })),

  finish: (outcome, failure, rollback) =>
    set({ finished: true, outcome, failure, rollback }),

  reset: () => set({ ...EMPTY }),
  setDrawerHeight: (height) => set({ drawerHeight: height }),
  toggleDrawer: () => set((s) => ({ drawerOpen: !s.drawerOpen })),
  setDrawerOpen: (open) => set({ drawerOpen: open }),
}));
