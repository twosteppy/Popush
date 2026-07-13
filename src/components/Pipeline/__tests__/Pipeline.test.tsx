import { render, screen, cleanup } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { Pipeline } from '../index';
import { usePipelineStore } from '../../../store/pipeline';
import type { PipelineState, UserMessage } from '../../../types/generated';

const FAILED_STATE: PipelineState = {
  steps: [
    {
      step: 'check',
      state: { state: 'ok', summary: 'ready', duration_ms: 12n },
    },
    {
      step: 'commit',
      state: { state: 'ok', summary: '1 file', duration_ms: 8n },
    },
    {
      step: 'push',
      state: { state: 'ok', summary: 'pushed', duration_ms: 30n },
    },
    {
      step: 'pull',
      state: { state: 'ok', summary: 'pulled', duration_ms: 40n },
    },
    {
      step: 'build',
      state: {
        state: 'failed',
        summary: 'npm run build exited 1',
        duration_ms: 900n,
      },
    },
    { step: 'restart', state: { state: 'pending' } },
    { step: 'verify', state: { state: 'pending' } },
  ],
  finished: true,
  rollback_sha: 'abc1234',
};

const FAILURE: UserMessage = {
  headline: 'The build step did not complete',
  consequence: 'Your site is still running the previous version.',
  next_action: { kind: 'advice', text: 'Check the build output above.' },
};

const ROLLBACK: UserMessage = {
  headline: 'Roll back to the previous version?',
  consequence: 'This restores the last working deploy.',
  next_action: { kind: 'run_command', command: 'git reset --hard abc1234' },
};

describe('Pipeline failed render', () => {
  beforeEach(() => {
    usePipelineStore.getState().reset();
  });
  afterEach(cleanup);

  it('renders the failure UserMessage headline, expanded output, and rollback', () => {
    const store = usePipelineStore.getState();
    store.begin('pipe-1');
    store.plan(FAILED_STATE);
    store.appendOutput(4, 'error: build failed on line 42');
    store.finish('failed', FAILURE, ROLLBACK);

    render(<Pipeline />);

    expect(
      screen.getByText('The build step did not complete'),
    ).toBeInTheDocument();
    expect(
      screen.getByText('Your site is still running the previous version.'),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/error: build failed on line 42/),
    ).toBeInTheDocument();
    expect(
      screen.getByText('Roll back to the previous version?'),
    ).toBeInTheDocument();
    expect(screen.getByText('git reset --hard abc1234')).toBeInTheDocument();
  });
});
