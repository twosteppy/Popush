import type { SiteStatus } from '../../types/generated';

export type StatusToken =
  'running' | 'stopped' | 'failed' | 'unknown' | 'working';

export interface StatusDescriptor {
  /** Semantic status token → drives the `bg-status-*` / colour var. */
  token: StatusToken;
  /** Text label; always rendered so colour is never the only signal. */
  label: string;
}

const COLOR_VAR: Record<StatusToken, string> = {
  running: 'var(--status-running)',
  stopped: 'var(--status-stopped)',
  failed: 'var(--status-failed)',
  unknown: 'var(--status-unknown)',
  working: 'var(--status-working)',
};

/** The CSS colour value for a token, for inline styling of dots/ticks. */
export function statusColor(token: StatusToken): string {
  return COLOR_VAR[token];
}

/** Describe a SiteStatus for rendering. */
export function describeSiteStatus(status: SiteStatus): StatusDescriptor {
  switch (status.state) {
    case 'running':
      return { token: 'running', label: 'Running' };
    case 'stopped':
      return { token: 'stopped', label: 'Stopped' };
    case 'failed':
      return { token: 'failed', label: 'Failed' };
    case 'checking':
      return { token: 'working', label: 'Checking' };
    case 'unknown':
      return { token: 'unknown', label: 'Unknown' };
  }
}
