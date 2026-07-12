// StatusPill — a status shown as a coloured dot + a WORD, wrapped in a subtle
// tinted chip (§20: colour is never the only signal, so the label always
// renders). The in-progress ("working") dot pulses, disabled under
// prefers-reduced-motion by the global rule and the motion-safe: gate.
//
// D14: presentation only. It names the status the backend already reported.

import type { SiteStatus } from '../../types/generated';
import { cn } from '../../lib/cn';
import {
  describeSiteStatus,
  statusColor,
  type StatusDescriptor,
} from './StatusLabel';

interface StatusPillProps {
  status?: SiteStatus;
  descriptor?: StatusDescriptor;
  /** Larger variant for page headers. */
  size?: 'sm' | 'md';
  className?: string;
}

export function StatusPill({
  status,
  descriptor,
  size = 'sm',
  className,
}: StatusPillProps) {
  const desc: StatusDescriptor =
    descriptor ??
    (status
      ? describeSiteStatus(status)
      : { token: 'unknown', label: 'Unknown' });
  const color = statusColor(desc.token);
  const isWorking = desc.token === 'working';

  return (
    <span
      role="status"
      className={cn(
        'inline-flex items-center gap-2 rounded-full border font-medium',
        size === 'md' ? 'px-3 py-1 text-sm' : 'px-2.5 py-0.5 text-xs',
        className,
      )}
      style={{
        color,
        borderColor: `${color}55`,
        backgroundColor: `${color}14`,
      }}
    >
      <span
        aria-hidden="true"
        className={cn(
          'inline-block shrink-0 rounded-full',
          size === 'md' ? 'h-2 w-2' : 'h-1.5 w-1.5',
          isWorking && 'motion-safe:animate-status-pulse',
        )}
        style={{ backgroundColor: color }}
      />
      {desc.label}
    </span>
  );
}
