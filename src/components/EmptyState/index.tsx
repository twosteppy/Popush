import { motion, useReducedMotion } from 'framer-motion';
import { Wand2, Plus, HelpCircle, ChevronRight } from 'lucide-react';
import type { SiteStatus } from '../../types/generated';
import { Logo } from '../ui/Logo';
import { Button } from '../ui/Button';
import { StatusPill } from '../ui/StatusPill';

/** One site summarised for the overview grid. */
export interface OverviewSite {
  id: string;
  label: string;
  serverLabel: string;
  status?: SiteStatus;
}

interface EmptyStateProps {
  /** True when servers exist but none/no site is selected. */
  hasServers: boolean;
  onAddServer: () => void;
  onRunWizard: () => void;
  /** Opens the "What is Popush?" explainer. Optional so callers can omit it. */
  onOpenHelp?: () => void;
  /** Every known site, shown as an at-a-glance overview when nothing is picked. */
  overview?: OverviewSite[];
  /** Open a site from the overview. */
  onSelectSite?: (id: string) => void;
}

export function EmptyState({
  hasServers,
  onAddServer,
  onRunWizard,
  onOpenHelp,
  overview,
  onSelectSite,
}: EmptyStateProps) {
  const reduce = useReducedMotion();

  if (hasServers) {
    const sites = overview ?? [];
    const online = sites.filter((s) => s.status?.state === 'running').length;
    return (
      <div className="mx-auto flex max-w-3xl flex-col gap-5 p-6">
        <div className="flex items-baseline justify-between">
          <h1 className="font-display text-xl font-semibold tracking-tight text-text-primary">
            Your sites
          </h1>
          {sites.length > 0 ? (
            <span className="label-mono text-[11px] text-text-tertiary">
              {online} of {sites.length} online
            </span>
          ) : null}
        </div>

        {sites.length === 0 ? (
          <p className="text-sm text-text-secondary">
            Pick a server in the sidebar, then add a site to deploy it.
          </p>
        ) : (
          <div className="grid grid-cols-1 gap-2.5 sm:grid-cols-2">
            {sites.map((s) => (
              <button
                key={s.id}
                type="button"
                onClick={() => onSelectSite?.(s.id)}
                className="lift-card group flex items-center justify-between gap-3 rounded-lg border-2 border-border-strong bg-surface-raised p-4 text-left shadow-hard-sm transition-colors hover:border-accent"
              >
                <div className="min-w-0">
                  <p className="truncate font-display font-semibold text-text-primary">
                    {s.label}
                  </p>
                  <p className="truncate text-xs text-text-tertiary">
                    {s.serverLabel}
                  </p>
                </div>
                <div className="flex shrink-0 items-center gap-1.5">
                  <StatusPill status={s.status} />
                  <ChevronRight
                    size={15}
                    aria-hidden="true"
                    className="text-text-tertiary transition-transform group-hover:translate-x-0.5"
                  />
                </div>
              </button>
            ))}
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="flex h-full items-center justify-center p-8">
      <motion.div
        initial={reduce ? false : { opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.25, ease: 'easeOut' }}
        className="flex w-full max-w-md flex-col items-center text-center"
      >
        <Logo size={44} />
        <h1 className="mt-6 font-display text-2xl font-semibold tracking-tight text-text-primary">
          Your VPS, one click away.
        </h1>
        <p className="mt-2 max-w-sm text-sm leading-relaxed text-text-secondary">
          Popush deploys your sites straight from this machine over SSH. No
          account, no Popush server, nothing leaves your computer.
        </p>

        <Button
          variant="primary"
          onClick={onAddServer}
          className="mt-7 h-[40px] px-5 text-[15px]"
        >
          <Plus size={16} aria-hidden="true" />
          Add your first server
        </Button>

        {onOpenHelp ? (
          <button
            type="button"
            onClick={onOpenHelp}
            className="pressable mt-5 inline-flex items-center gap-1.5 rounded-sm border border-border-strong px-3 py-1.5 text-xs text-text-secondary shadow-hard-sm hover:bg-surface-hover hover:text-text-primary"
          >
            <HelpCircle size={13} aria-hidden="true" />
            New here? See how Popush works
          </button>
        ) : null}

        <div className="mt-6 flex flex-col items-center gap-2 text-xs text-text-tertiary">
          <button
            type="button"
            onClick={onRunWizard}
            className="inline-flex items-center gap-1.5 rounded-sm px-2 py-1 hover:text-text-secondary"
          >
            <Wand2 size={13} aria-hidden="true" />
            Run the setup wizard
          </button>
        </div>
      </motion.div>
    </div>
  );
}
