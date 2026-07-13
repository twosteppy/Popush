// Sidebar (§14.2 left region, fixed ~230px). SERVERS and SITES sections with
// status dots + labels, an "Add server" affordance, and Help / Wizard /
// Settings entries.
//
// D14: renders state and dispatches selection/navigation intents only.

import { HelpCircle, Plus, Settings, Wand2 } from 'lucide-react';
import type { Panel } from '../../App';
import { useServersStore } from '../../store/servers';
import { useSitesStore } from '../../store/sites';
import { StatusDot } from '../StatusDot';
import { cn } from '../../lib/cn';

interface SidebarProps {
  activePanel: Panel;
  onOpenSettings: () => void;
  onOpenWizard: () => void;
  onOpenHelp: () => void;
  onAddServer: () => void;
  onSelectSite: () => void;
}

export function Sidebar({
  activePanel,
  onOpenSettings,
  onOpenWizard,
  onOpenHelp,
  onAddServer,
  onSelectSite,
}: SidebarProps) {
  const { servers, selectedServerId, select: selectServer } = useServersStore();
  const {
    sitesByServer,
    statusBySite,
    selectedSiteId,
    select: selectSite,
  } = useSitesStore();

  const sites = selectedServerId ? (sitesByServer[selectedServerId] ?? []) : [];

  return (
    <nav
      aria-label="Servers and sites"
      className="flex h-full w-[230px] shrink-0 flex-col border-r border-border-strong bg-surface-raised"
    >
      <div className="flex-1 overflow-y-auto px-2.5 py-3">
        <Section
          title="Servers"
          action={
            <button
              type="button"
              onClick={onAddServer}
              aria-label="Add server"
              className="inline-flex h-5 w-5 items-center justify-center rounded-sm border border-transparent text-text-tertiary transition-colors hover:border-border-subtle hover:bg-surface-hover hover:text-text-secondary"
            >
              <Plus size={13} aria-hidden="true" />
            </button>
          }
        >
          {servers.length === 0 ? (
            <EmptyLine>No servers yet</EmptyLine>
          ) : (
            servers.map((server) => (
              <button
                key={server.id}
                type="button"
                onClick={() => selectServer(server.id)}
                aria-current={server.id === selectedServerId || undefined}
                className={rowClass(server.id === selectedServerId)}
              >
                <StatusDot
                  descriptor={{ token: 'unknown', label: server.label }}
                  showLabel={false}
                />
                <span className="truncate">{server.label}</span>
              </button>
            ))
          )}
        </Section>

        <Section title="Sites">
          {sites.length === 0 ? (
            <EmptyLine>
              {selectedServerId
                ? 'No sites on this server'
                : 'No server selected'}
            </EmptyLine>
          ) : (
            sites.map((site) => {
              const status = statusBySite[site.id];
              const active =
                site.id === selectedSiteId && activePanel === 'site';
              return (
                <button
                  key={site.id}
                  type="button"
                  onClick={() => {
                    selectSite(site.id);
                    onSelectSite();
                  }}
                  aria-current={active || undefined}
                  className={rowClass(active)}
                >
                  <StatusDot status={status} showLabel={false} />
                  <span className="truncate">{site.label}</span>
                </button>
              );
            })
          )}
        </Section>
      </div>

      <div className="flex flex-col gap-0.5 border-t border-border-strong px-2.5 py-2">
        <button
          type="button"
          onClick={onOpenHelp}
          aria-current={activePanel === 'help' || undefined}
          className={rowClass(activePanel === 'help')}
        >
          <HelpCircle size={14} aria-hidden="true" className="shrink-0" />
          How it works
        </button>
        <button
          type="button"
          onClick={onOpenWizard}
          aria-current={activePanel === 'wizard' || undefined}
          className={rowClass(activePanel === 'wizard')}
        >
          <Wand2 size={14} aria-hidden="true" className="shrink-0" />
          Setup wizard
        </button>
        <button
          type="button"
          onClick={onOpenSettings}
          aria-current={activePanel === 'settings' || undefined}
          className={rowClass(activePanel === 'settings')}
        >
          <Settings size={14} aria-hidden="true" className="shrink-0" />
          Settings
        </button>
      </div>
    </nav>
  );
}

function rowClass(active: boolean): string {
  return cn(
    'flex w-full items-center gap-2.5 rounded-sm border px-2 py-1.5 text-left text-sm transition-colors',
    active
      ? 'border-border-strong bg-surface-hover text-text-primary shadow-hard-sm'
      : 'border-transparent text-text-secondary hover:border-border-subtle hover:bg-surface-hover hover:text-text-primary',
  );
}

function Section({
  title,
  action,
  children,
}: {
  title: string;
  action?: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <section className="mb-4">
      <div className="mb-1 flex items-center justify-between px-2">
        <h2 className="label-mono text-[10px] font-semibold text-text-tertiary">
          {title}
        </h2>
        {action}
      </div>
      <div className="flex flex-col gap-0.5">{children}</div>
    </section>
  );
}

function EmptyLine({ children }: { children: React.ReactNode }) {
  return <p className="px-2 py-1 text-xs text-text-tertiary">{children}</p>;
}
