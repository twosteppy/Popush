// GitPanel renders the changed-files list (with change kind + staged state) and
// the branch/ahead-behind header, and falls back to a clean-tree empty state
// when there are no changes or no status (e.g. running outside Tauri).

import { render, screen, cleanup } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { GitPanel } from '../index';
import type { GitStatus } from '../../../types/generated';

const DIRTY: GitStatus = {
  branch: 'main',
  ahead: 2,
  behind: 0,
  changed_files: [
    { path: 'src/App.tsx', change: 'modified', staged: true },
    { path: 'src/new.ts', change: 'untracked', staged: false },
  ],
  has_conflicts: false,
  remote_url: 'git@github.com:me/repo.git',
  remote_is_ssh: true,
};

const CLEAN: GitStatus = { ...DIRTY, ahead: 0, changed_files: [] };

const noop = () => {};

describe('GitPanel', () => {
  afterEach(cleanup);

  it('renders changed files with branch and staged state', () => {
    render(
      <GitPanel
        status={DIRTY}
        selected={new Set()}
        onToggle={noop}
        message=""
        onMessageChange={noop}
      />,
    );
    expect(screen.getByText('src/App.tsx')).toBeInTheDocument();
    expect(screen.getByText('src/new.ts')).toBeInTheDocument();
    expect(screen.getByText('main')).toBeInTheDocument();
    expect(screen.getByText('staged')).toBeInTheDocument();
    expect(screen.getByText('unstaged')).toBeInTheDocument();
  });

  it('shows the clean-tree state when there are no changes', () => {
    render(
      <GitPanel
        status={CLEAN}
        selected={new Set()}
        onToggle={noop}
        message=""
        onMessageChange={noop}
      />,
    );
    expect(screen.getByText(/Working tree is clean/)).toBeInTheDocument();
  });

  it('falls back to the clean-tree state when status is null', () => {
    render(
      <GitPanel
        status={null}
        selected={new Set()}
        onToggle={noop}
        message=""
        onMessageChange={noop}
      />,
    );
    expect(screen.getByText(/Working tree is clean/)).toBeInTheDocument();
  });
});
