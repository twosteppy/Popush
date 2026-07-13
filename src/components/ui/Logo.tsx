// Popush mark + wordmark. The mark is a rounded-square glyph in the violet
// accent (the one saturated brand colour permitted alongside status, D15); the
// wordmark is plain type. Sizes scale together.

import { cn } from '../../lib/cn';

interface LogoProps {
  /** Pixel size of the square mark. Wordmark scales relative to it. */
  size?: number;
  /** Render only the mark (no wordmark). */
  markOnly?: boolean;
  className?: string;
}

export function Logo({ size = 22, markOnly = false, className }: LogoProps) {
  return (
    <span className={cn('inline-flex items-center gap-2.5', className)}>
      <span
        aria-hidden="true"
        className="relative inline-flex shrink-0 items-center justify-center rounded-sm border border-accent bg-accent text-text-inverse shadow-hard-sm"
        style={{ width: size, height: size }}
      >
        <svg
          width={size * 0.58}
          height={size * 0.58}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth={3}
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          {/* An upward "push"/ship chevron. */}
          <path d="M12 19V7" />
          <path d="M6 11l6-6 6 6" />
        </svg>
      </span>
      {markOnly ? null : (
        <span
          className="font-display font-semibold tracking-tight text-text-primary"
          style={{ fontSize: size * 0.82 }}
        >
          Popush
        </span>
      )}
    </span>
  );
}
