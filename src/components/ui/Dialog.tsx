// Radix Dialog wrapper (§20). Escape closes, focus is trapped, and the overlay
// is greyscale (D15). Content is theme-token driven. A subtle fade + small
// slide (≤200ms, ease-out) plays on open via framer-motion, and is fully
// disabled under prefers-reduced-motion (useReducedMotion).

import * as RadixDialog from '@radix-ui/react-dialog';
import { motion, useReducedMotion } from 'framer-motion';
import type { ReactNode } from 'react';
import { cn } from '../../lib/cn';

interface DialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  /** Optional description read by screen readers (and shown unless hidden). */
  description?: string;
  children: ReactNode;
  /** Hide the visible title (still available to assistive tech). */
  hideTitle?: boolean;
  /** Content max width. */
  size?: 'md' | 'lg';
  /** Optional sticky footer (e.g. form actions). */
  footer?: ReactNode;
}

export function Dialog({
  open,
  onOpenChange,
  title,
  description,
  children,
  hideTitle,
  size = 'md',
  footer,
}: DialogProps) {
  const reduce = useReducedMotion();

  return (
    <RadixDialog.Root open={open} onOpenChange={onOpenChange}>
      <RadixDialog.Portal>
        <RadixDialog.Overlay asChild>
          <motion.div
            className="fixed inset-0 z-40 bg-black/70"
            initial={reduce ? false : { opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.15, ease: 'easeOut' }}
          />
        </RadixDialog.Overlay>
        <RadixDialog.Content asChild>
          <motion.div
            initial={reduce ? false : { opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.16, ease: 'easeOut' }}
            /* Capped so it can never overflow the fixed 1080x720 window: width
             * min(cap, viewport - 64px), height viewport - 64px with the body
             * scrolling internally. Centred on both axes, hard offset shadow. */
            className={cn(
              'fixed left-1/2 top-1/2 z-50 flex max-h-[calc(100vh-4rem)] -translate-x-1/2 -translate-y-1/2 flex-col overflow-hidden rounded-lg border-2 border-border-strong bg-surface-overlay shadow-hard focus:outline-none',
              size === 'lg'
                ? 'w-[min(560px,calc(100vw-4rem))]'
                : 'w-[min(460px,calc(100vw-4rem))]',
            )}
          >
            <div className="flex-1 overflow-y-auto p-5">
              <RadixDialog.Title
                className={
                  hideTitle
                    ? 'sr-only'
                    : 'font-display text-sm font-semibold uppercase tracking-wide text-text-primary'
                }
              >
                {title}
              </RadixDialog.Title>
              {description ? (
                <RadixDialog.Description className="mt-1 text-sm text-text-secondary">
                  {description}
                </RadixDialog.Description>
              ) : null}
              <div className={hideTitle ? '' : 'mt-4'}>{children}</div>
            </div>
            {footer ? (
              <div className="flex shrink-0 items-center justify-end gap-2 border-t border-border-strong bg-surface-raised px-5 py-3">
                {footer}
              </div>
            ) : null}
          </motion.div>
        </RadixDialog.Content>
      </RadixDialog.Portal>
    </RadixDialog.Root>
  );
}
