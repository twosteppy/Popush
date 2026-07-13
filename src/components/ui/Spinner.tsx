// Spinner - a small themed loading indicator built on lucide's Loader2, tinted
// with the pink accent (D15). It rotates via Tailwind's animate-spin; under
// prefers-reduced-motion the app's global rule (styles/globals.css) stops the
// rotation so it settles into a static ring, still paired with a text label at
// each call site. It is decorative (aria-hidden); the surrounding label or the
// button caption carries the accessible status.
//
// D14: presentation only.

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
