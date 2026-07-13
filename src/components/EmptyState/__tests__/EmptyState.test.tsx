import { render, screen, fireEvent, cleanup } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { EmptyState } from '../index';

describe('EmptyState onboarding', () => {
  afterEach(cleanup);

  it('renders the tagline and a primary "Add your first server" CTA', () => {
    const onAddServer = vi.fn();
    render(
      <EmptyState
        hasServers={false}
        onAddServer={onAddServer}
        onRunWizard={() => {}}
      />,
    );

    expect(screen.getByText('Your VPS, one click away.')).toBeInTheDocument();

    const cta = screen.getByRole('button', { name: /Add your first server/ });
    fireEvent.click(cta);
    expect(onAddServer).toHaveBeenCalledTimes(1);
  });

  it('links to the "how it works" explainer when a handler is provided', () => {
    const onOpenHelp = vi.fn();
    render(
      <EmptyState
        hasServers={false}
        onAddServer={() => {}}
        onRunWizard={() => {}}
        onOpenHelp={onOpenHelp}
      />,
    );
    fireEvent.click(
      screen.getByRole('button', { name: /See how Popush works/i }),
    );
    expect(onOpenHelp).toHaveBeenCalledTimes(1);
  });

  it('offers the setup-wizard secondary action', () => {
    const onRunWizard = vi.fn();
    render(
      <EmptyState
        hasServers={false}
        onAddServer={() => {}}
        onRunWizard={onRunWizard}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: /setup wizard/i }));
    expect(onRunWizard).toHaveBeenCalledTimes(1);
  });

  it('shows a sites overview once servers exist', () => {
    render(
      <EmptyState
        hasServers
        onAddServer={() => {}}
        onRunWizard={() => {}}
        overview={[
          {
            id: 's1',
            label: 'My Site',
            serverLabel: 'My VPS',
            status: { state: 'running', since: null },
          },
        ]}
        onSelectSite={() => {}}
      />,
    );
    expect(screen.getByText('Your sites')).toBeInTheDocument();
    expect(screen.getByText('My Site')).toBeInTheDocument();
    expect(screen.getByText('1 of 1 online')).toBeInTheDocument();
    expect(
      screen.queryByRole('button', { name: /Add your first server/ }),
    ).toBeNull();
  });

  it('opens a site when its overview card is clicked', () => {
    const onSelectSite = vi.fn();
    render(
      <EmptyState
        hasServers
        onAddServer={() => {}}
        onRunWizard={() => {}}
        overview={[{ id: 's1', label: 'My Site', serverLabel: 'My VPS' }]}
        onSelectSite={onSelectSite}
      />,
    );
    fireEvent.click(screen.getByText('My Site'));
    expect(onSelectSite).toHaveBeenCalledWith('s1');
  });
});
