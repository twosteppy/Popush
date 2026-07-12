// §20: colour is never the only signal. A StatusDot must always render a text
// label alongside the coloured dot.

import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { StatusDot } from '../components/StatusDot';
import type { SiteStatus } from '../types/generated';

describe('StatusDot', () => {
  it('renders a text label for a running status', () => {
    const running: SiteStatus = { state: 'running', since: null };
    render(<StatusDot status={running} />);
    expect(screen.getByText('Running')).toBeInTheDocument();
  });

  it('renders a text label for a failed status', () => {
    const failed: SiteStatus = { state: 'failed', reason: 'boom' };
    render(<StatusDot status={failed} />);
    expect(screen.getByText('Failed')).toBeInTheDocument();
  });

  it('keeps the label available even when visually hidden', () => {
    const stopped: SiteStatus = { state: 'stopped' };
    render(<StatusDot status={stopped} showLabel={false} />);
    // Still in the accessibility tree (sr-only), never colour-only.
    expect(screen.getByText('Stopped')).toBeInTheDocument();
  });
});
