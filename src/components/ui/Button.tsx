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

const BASE =
  'inline-flex h-[34px] items-center justify-center gap-2 rounded-md px-3 text-sm font-medium transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:opacity-50';

const VARIANTS: Record<ButtonVariant, string> = {
  primary:
    'bg-accent text-text-inverse hover:bg-accent-hover disabled:hover:bg-accent',
  secondary:
    'bg-surface-raised text-text-primary border border-border-strong hover:bg-surface-hover',
  destructive:
    'bg-transparent text-status-failed border border-status-failed/40 hover:bg-status-failed/10',
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
