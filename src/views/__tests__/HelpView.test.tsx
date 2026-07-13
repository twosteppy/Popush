// The explainer view must render its skimmable section headings and dispatch
// the "add server" intent from its getting-started CTA.

import { render, screen, fireEvent, cleanup } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { HelpView } from '../HelpView';

describe('HelpView explainer', () => {
  afterEach(cleanup);

  it('renders the title and the main section headings', () => {
    render(<HelpView onAddServer={() => {}} />);
    expect(
      screen.getByRole('heading', { name: /What is Popush\?/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: /The key ideas/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: /Your data stays yours/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: /Getting started/ }),
    ).toBeInTheDocument();
  });

  it('defines the core vocabulary', () => {
    render(<HelpView onAddServer={() => {}} />);
    for (const term of ['Server', 'Site', 'Ship It']) {
      expect(screen.getByRole('heading', { name: term })).toBeInTheDocument();
    }
  });

  it('dispatches add-server from the getting-started CTA', () => {
    const onAddServer = vi.fn();
    render(<HelpView onAddServer={onAddServer} />);
    fireEvent.click(
      screen.getByRole('button', { name: /Add your first server/ }),
    );
    expect(onAddServer).toHaveBeenCalledTimes(1);
  });
});
