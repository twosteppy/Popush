import { useState } from 'react';
import { HelpCircle, Plus, Settings, Trash2, Wand2 } from 'lucide-react';
import type { Panel } from '../../App';
import type { SiteConfig, SiteStatus } from '../../types/generated';
import type { StatusDescriptor } from '../ui/StatusLabel';
import { useServersStore } from '../../store/servers';
import { useSitesStore } from '../../store/sites';
import { removeSite } from '../../lib/ipc';
import { StatusDot } from '../StatusDot';
import { Skeleton } from '../ui/Skeleton';
import { ConfirmDeleteDialog } from '../ConfirmDeleteDialog';
import { cn } from '../../lib/cn';

/** What the confirm dialog is currently targeting. */
type DeleteTarget =
  | { kind: 'server'; id: string; name: string; siteCount: number }
  | { kind: 'site'; id: string; serverId: string; name: string };

interface SidebarProps {
  activePanel: Panel;
  onOpenSettings: () => void;
  onOpenWizard: () => void;
  onOpenHelp: () => void;
  onAddServer: () => void;
  onAddSite: () => void;
  onSelectSite: () => void;
}

export function Sidebar({
  activePanel,
  onOpenSettings,
  onOpenWizard,
  onOpenHelp,
  onAddServer,
  onAddSite,
  onSelectSite,
}: SidebarProps) {
  const {
    servers,
    selectedServerId,
    loading,
    select: selectServer,
    remove: removeServer,
  } = useServersStore();
  const {
    sitesByServer,
    statusBySite,
    selectedSiteId,
    select: selectSite,
    refreshSites,
  } = useSitesStore();

  // A typed-name confirmation guards every delete, so nothing goes on a slip.
  const [target, setTarget] = useState<DeleteTarget | null>(null);

  async function confirmDelete() {
    if (!target) return;
    if (target.kind === 'server') {
      await removeServer(target.id);
    } else {
      await removeSite(target.id);
      if (selectedSiteId === target.id) selectSite(null);
      await refreshSites(target.serverId);
    }
  }

  const sites = selectedServerId ? (sitesByServer[selectedServerId] ?? []) : [];

  return (
    <nav
      aria-label="Servers and sites"
      className="flex h-full w-[230px] shrink-0 flex-col border-r border-border-strong bg-surface-raised"
    >
      <div className="flex-1 overflow-y-auto px-2.5 py-3">
        <Section title="Servers">
          {loading && servers.length === 0 ? (
            <ListSkeleton rows={3} />
          ) : servers.length === 0 ? (
            <EmptyLine>No servers yet</EmptyLine>
          ) : (
            servers.map((server) => (
              <Row
                key={server.id}
                active={server.id === selectedServerId}
                label={server.label}
                onSelect={() => selectServer(server.id)}
                onRemove={() =>
                  setTarget({
                    kind: 'server',
                    id: server.id,
                    name: server.label,
                    siteCount: (sitesByServer[server.id] ?? []).length,
                  })
                }
              >
                <StatusDot
                  descriptor={serverDescriptor(
                    sitesByServer[server.id] ?? [],
                    statusBySite,
                  )}
                  showLabel={false}
                />
                <span className="truncate">{server.label}</span>
              </Row>
            ))
          )}
          <AddButton label="Add server" onClick={onAddServer} />
        </Section>

        <Section title="Sites">
          {loading && servers.length === 0 ? (
            <ListSkeleton rows={2} />
          ) : sites.length === 0 ? (
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
                <Row
                  key={site.id}
                  active={active}
                  label={site.label}
                  onSelect={() => {
                    selectSite(site.id);
                    onSelectSite();
                  }}
                  onRemove={() =>
                    setTarget({
                      kind: 'site',
                      id: site.id,
                      serverId: selectedServerId as string,
                      name: site.label,
                    })
                  }
                >
                  <StatusDot status={status} showLabel={false} />
                  <span className="truncate">{site.label}</span>
                </Row>
              );
            })
          )}
          {selectedServerId ? (
            <AddButton label="Add site" onClick={onAddSite} />
          ) : null}
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

      <ConfirmDeleteDialog
        open={target !== null}
        onOpenChange={(open) => {
          if (!open) setTarget(null);
        }}
        kind={target?.kind ?? 'site'}
        name={target?.name ?? ''}
        consequence={
          target?.kind === 'server' && target.siteCount > 0
            ? `Its ${target.siteCount} site${target.siteCount === 1 ? '' : 's'} will be removed too.`
            : undefined
        }
        onConfirm={confirmDelete}
      />
    </nav>
  );
}

/**
 * A server's dot follows its sites: green while anything on it is online,
 * red when everything is down, pulsing until the first check lands.
 */
function serverDescriptor(
  sites: SiteConfig[],
  statusBySite: Record<string, SiteStatus>,
): StatusDescriptor {
  const known = sites
    .map((site) => statusBySite[site.id])
    .filter((s): s is SiteStatus => Boolean(s));
  if (sites.length === 0 || known.length === 0) {
    return { token: 'working', label: 'Checking' };
  }
  return known.some((s) => s.state === 'running')
    ? { token: 'running', label: 'Online' }
    : { token: 'failed', label: 'Offline' };
}

function rowClass(active: boolean): string {
  return cn(
    'nav-row flex w-full items-center gap-2.5 rounded-sm border px-2 py-1.5 text-left text-sm transition-colors',
    active
      ? 'border-border-strong bg-surface-hover text-text-primary shadow-hard-sm'
      : 'border-transparent text-text-secondary hover:border-border-subtle hover:bg-surface-hover hover:text-text-primary',
  );
}

/**
 * A selectable row with a remove control that only appears on hover. The
 * trash opens a typed-name confirmation, so a slip cannot delete anything.
 */
function Row({
  active,
  label,
  onSelect,
  onRemove,
  children,
}: {
  active: boolean;
  label: string;
  onSelect: () => void;
  onRemove: () => void;
  children: React.ReactNode;
}) {
  return (
    <div
      className={cn(
        'nav-row group flex w-full items-center rounded-sm border text-sm transition-colors',
        active
          ? 'border-border-strong bg-surface-hover text-text-primary shadow-hard-sm'
          : 'border-transparent text-text-secondary hover:border-border-subtle hover:bg-surface-hover hover:text-text-primary',
      )}
    >
      <button
        type="button"
        onClick={onSelect}
        aria-current={active || undefined}
        className="flex min-w-0 flex-1 items-center gap-2.5 px-2 py-1.5 text-left"
      >
        {children}
      </button>
      <button
        type="button"
        onClick={onRemove}
        aria-label={`Remove ${label}`}
        title={`Remove ${label}`}
        className="mr-1 shrink-0 rounded-sm p-1 text-text-tertiary opacity-0 transition-all hover:text-status-failed focus-visible:opacity-100 group-hover:opacity-100"
      >
        <Trash2 size={12} aria-hidden="true" />
      </button>
    </div>
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

function AddButton({ label, onClick }: { label: string; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="mt-1 flex w-full items-center gap-2 rounded-sm border border-dashed border-border-strong px-2 py-1.5 text-left text-xs text-text-tertiary transition-colors hover:border-accent hover:bg-surface-hover hover:text-accent"
    >
      <Plus size={13} aria-hidden="true" className="shrink-0" />
      {label}
    </button>
  );
}

/** Placeholder rows shown while the sidebar list hydrates from the backend. */
function ListSkeleton({ rows }: { rows: number }) {
  return (
    <div className="flex flex-col gap-1 px-2 py-1" aria-hidden="true">
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} className="flex items-center gap-2.5">
          <Skeleton className="h-2 w-2 rounded-full" />
          <Skeleton className="h-3 flex-1" />
        </div>
      ))}
    </div>
  );
}
