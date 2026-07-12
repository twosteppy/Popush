// SiteCard — the header card for a site. Title (20px/600), url, and a
// "Deployed … · branch · sha" line. surface-raised, border-subtle, radius-lg.
// Selected state adds a 2px accent left border.
//
// D14: purely presentational; it renders the SiteConfig/SiteStatus it is given.

import type { SiteConfig, SiteStatus } from '../../types/generated';
import { StatusDot } from '../StatusDot';

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
    site.git_branch,
    sha ? sha.slice(0, 7) : null,
  ].filter((p): p is string => Boolean(p));

  return (
    <article
      className={`rounded-lg border border-border-subtle bg-surface-raised p-4 ${
        selected ? 'border-l-2 border-l-accent' : ''
      }`}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h1 className="truncate text-[20px] font-semibold text-text-primary">
            {site.label}
          </h1>
          {site.live_url ? (
            <a
              href={site.live_url}
              className="text-sm text-accent hover:text-accent-hover"
              target="_blank"
              rel="noreferrer"
            >
              {site.live_url}
            </a>
          ) : (
            <span className="text-sm text-text-tertiary">No public URL</span>
          )}
        </div>
        <StatusDot status={status} />
      </div>

      {metaParts.length > 0 ? (
        <p className="mt-3 font-mono text-xs text-text-secondary">
          {metaParts.join(' · ')}
        </p>
      ) : null}
    </article>
  );
}
