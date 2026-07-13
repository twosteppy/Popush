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
        className="pressable inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-sm border border-border-subtle bg-surface-raised text-text-secondary hover:border-border-strong hover:bg-surface-hover hover:text-text-primary focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        <X size={15} aria-hidden="true" />
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
              {/* Header bar. A bottom border separates it from the body so the
               * close button always sits alone in its own row, never against a
               * title or the search field. */}
              <div className="flex shrink-0 items-center justify-between gap-3 border-b border-border-strong px-4 py-3">
                {hideTitle ? (
                  <>
                    <span className="label-mono text-[10px] uppercase tracking-wide text-text-tertiary">
                      Search
                    </span>
                    <RadixDialog.Title className="sr-only">
                      {title}
                    </RadixDialog.Title>
                  </>
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

              <div className="flex-1 overflow-y-auto p-4">{children}</div>

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
