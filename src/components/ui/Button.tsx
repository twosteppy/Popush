// §14.5 button variants. Real <button> elements, visible focus rings, and a
// tooltip on disabled buttons explaining why they are disabled.
//
// primary:     filled accent + text-inverse
// secondary:   surface-raised + border-strong
// destructive: subtle red-tinted outline (NOT solid red)

import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from 'react';
import { Tooltip } from './Tooltip';

export type ButtonVariant = 'primary' | 'secondary' | 'destructive';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  /** When disabled, this text is shown in a tooltip explaining why. */
  disabledReason?: ReactNode;
  children: ReactNode;
}

// Chunky solid border + hard offset shadow that collapses on press (.pressable
// nudges the element 1px into its own shadow). Mono caption, slight tracking.
const BASE =
  'pressable inline-flex h-[34px] items-center justify-center gap-2 rounded-sm border px-3.5 font-display text-xs font-medium uppercase tracking-wider focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:opacity-50 disabled:shadow-none active:shadow-none';

const VARIANTS: Record<ButtonVariant, string> = {
  primary:
    'border-accent bg-accent text-text-inverse shadow-hard hover:bg-accent-hover disabled:hover:bg-accent',
  secondary:
    'border-border-strong bg-surface-raised text-text-primary shadow-hard-sm hover:bg-surface-hover',
  destructive:
    'border-status-failed/60 bg-transparent text-status-failed shadow-hard-sm hover:bg-status-failed/10',
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  function Button(
    {
      variant = 'secondary',
      disabledReason,
      disabled,
      className,
      children,
      ...rest
    },
    ref,
  ) {
    const button = (
      <button
        ref={ref}
        type="button"
        disabled={disabled}
        aria-disabled={disabled || undefined}
        className={`${BASE} ${VARIANTS[variant]} ${className ?? ''}`}
        {...rest}
      >
        {children}
      </button>
    );

    // A disabled button that renders carries a tooltip explaining why.
    if (disabled && disabledReason) {
      return (
        <Tooltip content={disabledReason}>
          {/* span wrapper so the tooltip still triggers over a disabled button */}
          <span className="inline-flex">{button}</span>
        </Tooltip>
      );
    }
    return button;
  },
);
