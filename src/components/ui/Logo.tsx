// Popush mark and wordmark. The mark is the app icon (src/assets/logo.png)
// rendered as a square glyph; the wordmark is plain type beside it. Sizes scale
// together.
//
// When `onClick` is supplied the whole lockup becomes a button (used in the
// header to return home). It carries a smooth hover: the mark lifts slightly and
// gains a soft pink glow, and the wordmark warms toward the accent. Everything
// respects prefers-reduced-motion through the shared transition tokens.

import { cn } from '../../lib/cn';
import logoUrl from '../../assets/logo.png';

interface LogoProps {
  /** Pixel size of the square mark. Wordmark scales relative to it. */
  size?: number;
  /** Render only the mark (no wordmark). */
  markOnly?: boolean;
  className?: string;
  /** When set, the lockup renders as a button with a hover treatment. */
  onClick?: () => void;
  /** Accessible label / tooltip for the interactive form. */
  label?: string;
}

export function Logo({
  size = 22,
  markOnly = false,
  className,
  onClick,
  label = 'Popush home',
}: LogoProps) {
  const interactive = typeof onClick === 'function';

  const inner = (
    <>
      <img
        src={logoUrl}
        alt=""
        aria-hidden="true"
        draggable={false}
        width={size}
        height={size}
        className={cn(
          'logo-mark shrink-0 select-none rounded-sm shadow-hard-sm',
          interactive && 'transition-transform duration-150 ease-out',
        )}
        style={{ width: size, height: size }}
      />
      {markOnly ? null : (
        <span
          className={cn(
            'logo-word font-display font-semibold tracking-tight text-text-primary',
            interactive && 'transition-colors duration-150 ease-out',
          )}
          style={{ fontSize: size * 0.82 }}
        >
          Popush
        </span>
      )}
    </>
  );

  if (interactive) {
    return (
      <button
        type="button"
        onClick={onClick}
        aria-label={label}
        title={label}
        className={cn(
          'logo-button group inline-flex items-center gap-2.5 rounded-sm outline-none',
          'focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent',
          className,
        )}
      >
        {inner}
      </button>
    );
  }

  return (
    <span className={cn('inline-flex items-center gap-2.5', className)}>
      {inner}
    </span>
  );
}
