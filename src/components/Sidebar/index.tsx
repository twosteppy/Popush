// Sidebar (§14.2 left region, ~240px). SERVERS and SITES sections with status
// dots, an "+ Add" affordance and a "Settings" entry, plus a footer/status bar
// that credits twostep (D9).
//
// D14: renders state and dispatches selection intents only.

import { Plus, Settings } from 'lucide-react';
import { useServersStore } from '../../store/servers';
import { useSitesStore } from '../../store/sites';
import { StatusDot } from '../StatusDot';

interface SidebarProps {
  onOpenSettings: () => void;
  onAdd: () => void;
}

export function Sidebar({ onOpenSettings, onAdd }: SidebarProps) {
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
      className="flex h-full w-60 shrink-0 flex-col border-r border-border-subtle bg-surface-raised"
    >
      <div className="flex-1 overflow-y-auto p-3">
        <Section title="Servers">
          {servers.length === 0 ? (
            <EmptyLine>No servers yet.</EmptyLine>
          ) : (
            servers.map((server) => (
              <button
                key={server.id}
                type="button"
                onClick={() => selectServer(server.id)}
                aria-current={server.id === selectedServerId || undefined}
                className={`flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-surface-hover ${
                  server.id === selectedServerId
                    ? 'bg-surface-hover text-text-primary'
                    : 'text-text-secondary'
                }`}
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
            <EmptyLine>No sites on this server.</EmptyLine>
          ) : (
            sites.map((site) => {
              const status = statusBySite[site.id];
              return (
                <button
                  key={site.id}
                  type="button"
                  onClick={() => selectSite(site.id)}
                  aria-current={site.id === selectedSiteId || undefined}
                  className={`flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-surface-hover ${
                    site.id === selectedSiteId
                      ? 'bg-surface-hover text-text-primary'
                      : 'text-text-secondary'
                  }`}
                >
                  <StatusDot status={status} showLabel={false} />
                  <span className="truncate">{site.label}</span>
                </button>
              );
            })
          )}
        </Section>

        <div className="mt-3 flex flex-col gap-1">
          <button
            type="button"
            onClick={onAdd}
            className="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm text-text-secondary hover:bg-surface-hover"
          >
            <Plus size={14} aria-hidden="true" />
            Add
          </button>
          <button
            type="button"
            onClick={onOpenSettings}
            className="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm text-text-secondary hover:bg-surface-hover"
          >
            <Settings size={14} aria-hidden="true" />
            Settings
          </button>
        </div>
      </div>

      {/* Footer / status bar — credits twostep (D9). */}
      <footer className="border-t border-border-subtle px-3 py-2 text-xs text-text-tertiary">
        Built by twostep
      </footer>
    </nav>
  );
}

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="mb-4">
      <h2 className="mb-1 px-2 text-xs font-semibold uppercase tracking-wide text-text-tertiary">
        {title}
      </h2>
      <div className="flex flex-col gap-0.5">{children}</div>
    </section>
  );
}

function EmptyLine({ children }: { children: React.ReactNode }) {
  return <p className="px-2 py-1 text-sm text-text-tertiary">{children}</p>;
}
