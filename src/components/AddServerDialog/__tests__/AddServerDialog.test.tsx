// Add Server dialog validation: required fields block advancing, and a valid
// connection advances to the optional-site step. The dialog lets users add a
// server without touching TOML.

import { render, screen, fireEvent, cleanup } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { AddServerDialog } from '../index';

function open() {
  render(<AddServerDialog open onOpenChange={() => {}} />);
}

describe('AddServerDialog validation', () => {
  afterEach(cleanup);

  it('blocks Continue and shows errors when required fields are empty', () => {
    open();
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));

    expect(screen.getByText('Host is required.')).toBeInTheDocument();
    expect(screen.getByText('Username is required.')).toBeInTheDocument();
    // Still on step 1 - the site step has not appeared.
    expect(screen.queryByText('Add a site to deploy now')).toBeNull();
  });

  it('rejects an out-of-range port on blur', () => {
    open();
    const port = screen.getByLabelText('Port');
    fireEvent.change(port, { target: { value: '70000' } });
    fireEvent.blur(port);
    expect(
      screen.getByText('Port must be between 1 and 65535.'),
    ).toBeInTheDocument();
  });

  it('advances to the optional site step once the connection is valid', () => {
    open();
    fireEvent.change(screen.getByLabelText('Name'), {
      target: { value: 'Production' },
    });
    fireEvent.change(screen.getByLabelText('Host'), {
      target: { value: '203.0.113.10' },
    });
    fireEvent.change(screen.getByLabelText('Username'), {
      target: { value: 'deploy' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));

    expect(screen.getByText('Add a site to deploy now')).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Add server' }),
    ).toBeInTheDocument();
  });

  it('reassures where the config is saved and that no secrets are stored', () => {
    open();
    expect(screen.getByText(/No secrets are stored/)).toBeInTheDocument();
  });
});
