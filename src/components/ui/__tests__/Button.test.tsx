// §20 + capability rules: a disabled button carries aria-disabled and cannot be
// clicked; an action whose capability is absent is simply NOT rendered by the
// ActionBar (never disable-and-fail).

import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Button } from '../Button';
import { ActionBar } from '../../ActionBar';
import type { Capabilities } from '../../../types/generated';

describe('Button', () => {
  it('renders a disabled button with aria and does not fire onClick', () => {
    const onClick = vi.fn();
    render(
      <Button disabled disabledReason="Busy" onClick={onClick}>
        Ship It
      </Button>,
    );
    const btn = screen.getByRole('button', { name: 'Ship It' });
    expect(btn).toBeDisabled();
    expect(btn).toHaveAttribute('aria-disabled', 'true');
    fireEvent.click(btn);
    expect(onClick).not.toHaveBeenCalled();
  });
});

describe('ActionBar capability gating', () => {
  const noop = () => {};

  it('hides Restart, Stop, and Logs when the adapter lacks them', () => {
    const caps: Capabilities = {
      can_start_stop: false,
      can_restart: false,
      has_logs: false,
      status_is_reliable: false,
    };
    render(
      <ActionBar
        capabilities={caps}
        onShipIt={noop}
        onRestart={noop}
        onStop={noop}
        onLogs={noop}
      />,
    );
    // Ship It always renders.
    expect(screen.getByRole('button', { name: /Ship It/ })).toBeInTheDocument();
    // Unsupported actions are absent, not merely disabled.
    expect(screen.queryByRole('button', { name: /Restart/ })).toBeNull();
    expect(screen.queryByRole('button', { name: /Stop/ })).toBeNull();
    expect(screen.queryByRole('button', { name: /Logs/ })).toBeNull();
  });

  it('renders supported actions', () => {
    const caps: Capabilities = {
      can_start_stop: true,
      can_restart: true,
      has_logs: true,
      status_is_reliable: true,
    };
    render(
      <ActionBar
        capabilities={caps}
        onShipIt={noop}
        onRestart={noop}
        onStop={noop}
        onLogs={noop}
      />,
    );
    expect(screen.getByRole('button', { name: /Restart/ })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Stop/ })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Logs/ })).toBeInTheDocument();
  });
});
