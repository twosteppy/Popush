// The log drawer must show a clean centered "Waiting for output…" state when
// there is no output (no terminal, no cursor artifacts), and reflect the last
// line cleanly on the collapsed bar once output arrives.

import { render, screen, cleanup } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { LogDrawer } from '../index';
import { usePipelineStore } from '../../../store/pipeline';
import type { PipelineState } from '../../../types/generated';

const PLAN: PipelineState = {
  steps: [{ step: 'build', state: { state: 'running' } }],
  finished: false,
  rollback_sha: null,
};

describe('LogDrawer', () => {
  beforeEach(() => usePipelineStore.getState().reset());
  afterEach(cleanup);

  it('shows the waiting state when open with no output', () => {
    usePipelineStore.getState().setDrawerOpen(true);
    render(<LogDrawer />);
    expect(screen.getByText('Waiting for output…')).toBeInTheDocument();
    // No terminal mounted while there is nothing to show.
    expect(screen.queryByTestId('log-terminal')).toBeNull();
  });

  it('shows "No output yet" on the collapsed bar before any lines', () => {
    render(<LogDrawer />);
    expect(screen.getByText('No output yet')).toBeInTheDocument();
  });

  it('surfaces the last streamed line on the collapsed bar', () => {
    const store = usePipelineStore.getState();
    store.begin('pipe-1');
    store.plan(PLAN);
    store.appendOutput(0, 'compiling module A');
    store.appendOutput(0, 'compiling module B');
    render(<LogDrawer />);
    expect(screen.getByText('compiling module B')).toBeInTheDocument();
  });
});
