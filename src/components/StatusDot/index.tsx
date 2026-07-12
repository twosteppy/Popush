// StatusDot — an 8px dot coloured from a SiteStatus, ALWAYS paired with a text
// label (§20: colour is never the only signal). The in-progress ("working")
// state pulses ~1.5s; the pulse is disabled by prefers-reduced-motion via the
// global stylesheet rule, and we also gate the animation class defensively.

import type { SiteStatus } from '../../types/generated';
import {
  describeSiteStatus,
  statusColor,
  type StatusDescriptor,
} from '../ui/StatusLabel';

interface StatusDotProps {
  /** Either a full SiteStatus, or a pre-computed descriptor. */
  status?: SiteStatus;
  descriptor?: StatusDescriptor;
  /** Show the text label beside the dot. Defaults to true. */
  showLabel?: boolean;
  className?: string;
}

export function StatusDot({
  status,
  descriptor,
  showLabel = true,
  className,
}: StatusDotProps) {
  const desc: StatusDescriptor =
    descriptor ??
    (status
      ? describeSiteStatus(status)
      : { token: 'unknown', label: 'Unknown' });

  const isWorking = desc.token === 'working';

  return (
    <span
      className={`inline-flex items-center gap-2 ${className ?? ''}`}
      role="status"
    >
      <span
        aria-hidden="true"
        className={`inline-block h-2 w-2 shrink-0 rounded-full ${
          isWorking ? 'motion-safe:animate-status-pulse' : ''
        }`}
        style={{ backgroundColor: statusColor(desc.token) }}
      />
      {/* Text label always present; when hidden visually it stays for a11y. */}
      <span className={showLabel ? 'text-sm text-text-secondary' : 'sr-only'}>
        {desc.label}
      </span>
    </span>
  );
}
