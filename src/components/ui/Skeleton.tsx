// Skeleton - a themed placeholder block for content that is still loading (e.g.
// the sidebar server/site list while list_servers resolves). It is a muted
// surface tile with a soft accent-tinted shimmer that sweeps across it. The
// shimmer is gated behind motion-safe, so under prefers-reduced-motion it is a
// plain static block (no sweep). Hard, pixel-leaning corners match the retro
// look (D15).
//
// D14: presentation only.

import { cn } from '../../lib/cn';

interface SkeletonProps {
  /** Extra classes to size the block (width/height/margins). */
  className?: string;
}

export function Skeleton({ className }: SkeletonProps) {
  return (
    <div
      aria-hidden="true"
      className={cn(
        'relative overflow-hidden rounded-sm bg-surface-hover',
        className,
      )}
    >
      <div className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-accent-muted to-transparent motion-safe:animate-shimmer" />
    </div>
  );
}
