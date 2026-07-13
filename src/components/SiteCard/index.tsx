// SiteCard - the header card for a site. Title (20px/600), live URL, a status
// pill (colour + word, §20), and a "Deployed … · branch · sha" metadata line.
// surface-raised, border-subtle, radius-lg; a selected site gets a 2px accent
// left edge.
//
// D14: purely presentational; it renders the SiteConfig/SiteStatus it is given.

import { ExternalLink, GitBranch } from 'lucide-react';
import type { SiteConfig, SiteStatus } from '../../types/generated';
import { StatusPill } from '../ui/StatusPill';

interface SiteCardProps {
  site: SiteConfig;
  status?: SiteStatus;
  selected?: boolean;
  /** Optional deploy metadata to show on the sub-line. */
  deployedAt?: string | null;
  sha?: string | null;
}

export function SiteCard({
  site,
  status,
  selected,
  deployedAt,
  sha,
}: SiteCardProps) {
  const metaParts = [
    deployedAt ? `Deployed ${deployedAt}` : null,
    sha ? sha.slice(0, 7) : null,
  ].filter((p): p is string => Boolean(p));

  return (
    <article
      className={`rounded-lg border-2 bg-surface-raised p-5 shadow-hard ${
        selected
          ? 'border-border-strong border-l-4 border-l-accent'
          : 'border-border-strong'
      }`}
    >
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0">
          <h1 className="truncate font-display text-xl font-semibold tracking-tight text-text-primary">
            {site.label}
          </h1>
          {site.live_url ? (
            <a
              href={site.live_url}
              className="mt-1 inline-flex items-center gap-1.5 text-sm text-accent hover:text-accent-hover"
              target="_blank"
              rel="noreferrer"
            >
              {site.live_url}
              <ExternalLink size={13} aria-hidden="true" />
            </a>
          ) : (
            <span className="mt-1 block text-sm text-text-tertiary">
              No public URL
            </span>
          )}
        </div>
        <StatusPill status={status} size="md" />
      </div>

      <div className="mt-4 flex flex-wrap items-center gap-x-2 gap-y-1 font-mono text-xs text-text-secondary">
        <span className="inline-flex items-center gap-1.5">
          <GitBranch
            size={12}
            aria-hidden="true"
            className="text-text-tertiary"
          />
          {site.git_branch}
        </span>
        {metaParts.map((part) => (
          <span key={part} className="text-text-tertiary">
            · {part}
          </span>
        ))}
      </div>
    </article>
  );
}
