// A small loading indicator built on lucide's Loader2, tinted with the pink
// accent. It rotates via animate-spin; under prefers-reduced-motion the global
// rule stops the rotation. It is decorative (aria-hidden), so the surrounding
// label or button caption carries the accessible status.

import { Loader2 } from 'lucide-react';
import { cn } from '../../lib/cn';

interface SpinnerProps {
  /** Icon size in pixels. */
  size?: number;
  /** Extra classes (e.g. to override the accent tint). */
  className?: string;
}

export function Spinner({ size = 16, className }: SpinnerProps) {
  return (
    <Loader2
      size={size}
      aria-hidden="true"
      className={cn('shrink-0 animate-spin text-accent', className)}
    />
  );
}
