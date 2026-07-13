// A placeholder block for content that is still loading (e.g. the sidebar list
// while list_servers resolves). A muted surface tile with a soft accent-tinted
// shimmer; the shimmer is gated behind motion-safe, so prefers-reduced-motion
// leaves a plain static block.

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
