// Radix Dialog wrapper. Escape closes, focus is trapped, and the overlay frosts
// the background with a blur rather than a heavy dark scrim. A subtle fade plus
// a small slide plays on open via framer-motion, disabled under
// prefers-reduced-motion.
//
// The animating card must not rely on Tailwind -translate-* classes, because
// framer-motion writes an inline transform (for the y slide) that would
// override them and drop the card into the bottom-right quadrant. So the
// centring lives on a static Radix Content wrapper (fixed + translate(-50%)),
// and only the inner motion card owns the transform for its animation.

import * as RadixDialog from '@radix-ui/react-dialog';
import { motion, useReducedMotion } from 'framer-motion';
import { useEffect, type ReactNode } from 'react';
import { X } from 'lucide-react';
import { cn } from '../../lib/cn';
import { useModalStore } from '../../store/modals';

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

  // Register as an open modal while mounted so global shortcuts (Ctrl+K) can
  // keep a single modal open at a time.
  useEffect(() => {
    if (!open) return;
    const { register, unregister } = useModalStore.getState();
    register();
    return unregister;
  }, [open]);

  const closeButton = (
    <RadixDialog.Close asChild>
      <button
        type="button"
        aria-label="Close"
        className="pressable -mr-1 -mt-1 inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-sm text-text-secondary hover:bg-surface-hover hover:text-text-primary focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        <X size={16} aria-hidden="true" />
      </button>
    </RadixDialog.Close>
  );

  return (
    <RadixDialog.Root open={open} onOpenChange={onOpenChange}>
      <RadixDialog.Portal>
        <RadixDialog.Overlay asChild>
          <motion.div
            className="dialog-overlay fixed inset-0 z-40"
            initial={reduce ? false : { opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.15, ease: 'easeOut' }}
          />
        </RadixDialog.Overlay>
        <RadixDialog.Content asChild>
          {/* Static centring wrapper: fixed + translate(-50%,-50%). Capped so it
           * can never overflow the fixed 1080x720 window. The inner card owns
           * the animation. */}
          <div
            className={cn(
              'fixed left-1/2 top-1/2 z-50 -translate-x-1/2 -translate-y-1/2',
              size === 'lg'
                ? 'w-[min(560px,calc(100vw-4rem))]'
                : 'w-[min(460px,calc(100vw-4rem))]',
            )}
          >
            <motion.div
              initial={reduce ? false : { opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.16, ease: 'easeOut' }}
              className="relative flex max-h-[calc(100vh-4rem)] flex-col overflow-hidden rounded-lg border-2 border-border-strong bg-surface-overlay shadow-hard focus:outline-none"
            >
              {/* Header zone. The close button lives here in its own space, so it
               * never overlaps the title, a field, or its border. */}
              <div
                className={cn(
                  'flex shrink-0 items-start justify-between gap-3',
                  hideTitle ? 'px-4 pt-4' : 'px-5 pt-5',
                )}
              >
                {hideTitle ? (
                  <RadixDialog.Title className="sr-only">
                    {title}
                  </RadixDialog.Title>
                ) : (
                  <div className="min-w-0">
                    <RadixDialog.Title className="font-display text-sm font-semibold uppercase tracking-wide text-text-primary">
                      {title}
                    </RadixDialog.Title>
                    {description ? (
                      <RadixDialog.Description className="mt-1 text-sm text-text-secondary">
                        {description}
                      </RadixDialog.Description>
                    ) : null}
                  </div>
                )}
                {closeButton}
              </div>

              <div
                className={cn(
                  'flex-1 overflow-y-auto px-5 pb-5',
                  hideTitle ? 'pt-3' : 'pt-4',
                )}
              >
                {children}
              </div>

              {footer ? (
                <div className="flex shrink-0 items-center justify-end gap-2 border-t border-border-strong bg-surface-raised px-5 py-3">
                  {footer}
                </div>
              ) : null}
            </motion.div>
          </div>
        </RadixDialog.Content>
      </RadixDialog.Portal>
    </RadixDialog.Root>
  );
}
