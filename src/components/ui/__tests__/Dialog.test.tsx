// Fix 1 + Fix 3: the shared Dialog wrapper renders a themed Close button that
// closes via the same onOpenChange(false) path as Escape, and registers its
// open state in the modal registry so global shortcuts can enforce a single
// modal at a time.

import { render, screen, fireEvent, cleanup } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { Dialog } from '../Dialog';
import { isAnyModalOpen, useModalStore } from '../../../store/modals';

afterEach(() => {
  cleanup();
  useModalStore.setState({ openCount: 0 });
});

describe('Dialog', () => {
  it('renders a Close button that calls onOpenChange(false)', () => {
    const onOpenChange = vi.fn();
    render(
      <Dialog open onOpenChange={onOpenChange} title="Add a server">
        <p>Body</p>
      </Dialog>,
    );
    const close = screen.getByRole('button', { name: 'Close' });
    expect(close).toBeInTheDocument();
    fireEvent.click(close);
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it('registers itself as an open modal while open', () => {
    const { rerender } = render(
      <Dialog open onOpenChange={() => {}} title="Add a server">
        <p>Body</p>
      </Dialog>,
    );
    expect(isAnyModalOpen()).toBe(true);

    rerender(
      <Dialog open={false} onOpenChange={() => {}} title="Add a server">
        <p>Body</p>
      </Dialog>,
    );
    expect(isAnyModalOpen()).toBe(false);
  });
});
