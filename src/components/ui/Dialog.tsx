// Radix Dialog wrapper (§20). Escape closes, focus is trapped, and the overlay
// is greyscale (D15). Content is theme-token driven.

import * as RadixDialog from '@radix-ui/react-dialog';
import type { ReactNode } from 'react';

interface DialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  /** Optional description read by screen readers. */
  description?: string;
  children: ReactNode;
  /** Hide the visible title (still available to assistive tech). */
  hideTitle?: boolean;
}

export function Dialog({
  open,
  onOpenChange,
  title,
  description,
  children,
  hideTitle,
}: DialogProps) {
  return (
    <RadixDialog.Root open={open} onOpenChange={onOpenChange}>
      <RadixDialog.Portal>
        <RadixDialog.Overlay className="fixed inset-0 z-40 bg-black/50" />
        <RadixDialog.Content className="fixed left-1/2 top-1/4 z-50 w-full max-w-lg -translate-x-1/2 rounded-lg border border-border-strong bg-surface-overlay p-4 shadow-2xl focus:outline-none">
          <RadixDialog.Title
            className={
              hideTitle
                ? 'sr-only'
                : 'mb-2 text-base font-semibold text-text-primary'
            }
          >
            {title}
          </RadixDialog.Title>
          {description ? (
            <RadixDialog.Description className="mb-3 text-sm text-text-secondary">
              {description}
            </RadixDialog.Description>
          ) : null}
          {children}
        </RadixDialog.Content>
      </RadixDialog.Portal>
    </RadixDialog.Root>
  );
}
