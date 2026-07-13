import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { StatusDot } from '../components/StatusDot';
import type { SiteStatus } from '../types/generated';

describe('StatusDot', () => {
  it('shows Online for a running status', () => {
    const running: SiteStatus = { state: 'running', since: null };
    render(<StatusDot status={running} />);
    expect(screen.getByText('Online')).toBeInTheDocument();
  });

  it('shows Offline for a failed status', () => {
    const failed: SiteStatus = { state: 'failed', reason: 'boom' };
    render(<StatusDot status={failed} />);
    expect(screen.getByText('Offline')).toBeInTheDocument();
  });

  it('keeps the label available even when visually hidden', () => {
    const stopped: SiteStatus = { state: 'stopped' };
    render(<StatusDot status={stopped} showLabel={false} />);
    expect(screen.getByText('Offline')).toBeInTheDocument();
  });

  it('shows a pulsing Checking state before the first result', () => {
    render(<StatusDot />);
    expect(screen.getByText('Checking')).toBeInTheDocument();
  });
});
