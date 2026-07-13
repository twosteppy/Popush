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
