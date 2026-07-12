// App — the three-region layout (§14.2): left sidebar (~240px), main panel,
// and the bottom log drawer. Wires the Ctrl+K command palette and Ctrl+`
// drawer toggle, and hydrates the stores from the backend on mount.
//
// D14: the shell holds no deployment logic. It renders state from the stores
// and dispatches selection/navigation intents.

import { useEffect, useMemo, useState } from 'react';
import { Sidebar } from './components/Sidebar';
import { LogDrawer } from './components/LogDrawer';
import { CommandPalette, type PaletteItem } from './components/CommandPalette';
import { SiteView } from './views/SiteView';
import { SettingsView } from './views/SettingsView';
import { AboutView } from './views/AboutView';
import { CommandLogView } from './views/CommandLogView';
import { useServersStore } from './store/servers';
import { useSitesStore } from './store/sites';
import { usePipelineStore } from './store/pipeline';
import { usePipelineEvents } from './hooks/usePipelineEvents';
import type { Theme } from './types/generated';

type Panel = 'site' | 'settings' | 'about' | 'log';

const APP_VERSION = '0.1.0';

export function App() {
  const [panel, setPanel] = useState<Panel>('site');
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [theme, setTheme] = useState<Theme>('system');
  const [pollInterval, setPollInterval] = useState(30);
  const [patDismissed, setPatDismissed] = useState(false);

  const { servers, selectedServerId, refresh } = useServersStore();
  const {
    sitesByServer,
    selectedSiteId,
    refreshSites,
    select: selectSite,
  } = useSitesStore();
  const { toggleDrawer } = usePipelineStore();

  // Mirror the backend pipeline event stream into the pipeline store.
  usePipelineEvents();

  // Hydrate the server mirror on mount (§6.3: backend is authoritative).
  useEffect(() => {
    void refresh();
  }, [refresh]);

  // Load sites whenever the selected server changes.
  useEffect(() => {
    if (selectedServerId) void refreshSites(selectedServerId);
  }, [selectedServerId, refreshSites]);

  // Apply an explicit theme choice to the document root.
  useEffect(() => {
    const el = document.documentElement;
    if (theme === 'system') el.removeAttribute('data-theme');
    else el.setAttribute('data-theme', theme);
  }, [theme]);

  // Global keyboard shortcuts: Ctrl+K palette, Ctrl+` drawer.
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.ctrlKey && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setPaletteOpen((o) => !o);
      } else if (e.ctrlKey && e.key === '`') {
        e.preventDefault();
        toggleDrawer();
      }
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [toggleDrawer]);

  const sites = selectedServerId ? (sitesByServer[selectedServerId] ?? []) : [];
  const selectedSite = sites.find((s) => s.id === selectedSiteId) ?? null;

  const paletteItems: PaletteItem[] = useMemo(() => {
    const items: PaletteItem[] = [];
    for (const [serverId, serverSites] of Object.entries(sitesByServer)) {
      for (const site of serverSites) {
        items.push({
          id: `site:${serverId}:${site.id}`,
          label: site.label,
          kind: 'Site',
          onSelect: () => {
            selectSite(site.id);
            setPanel('site');
          },
        });
      }
    }
    items.push({
      id: 'action:settings',
      label: 'Open Settings',
      kind: 'Action',
      onSelect: () => setPanel('settings'),
    });
    items.push({
      id: 'action:log',
      label: 'View Command Log',
      kind: 'Action',
      onSelect: () => setPanel('log'),
    });
    items.push({
      id: 'action:about',
      label: 'About Popush',
      kind: 'Action',
      onSelect: () => setPanel('about'),
    });
    return items;
  }, [sitesByServer, selectSite]);

  return (
    <div className="flex h-screen w-screen flex-col bg-surface-base text-text-primary">
      <div className="flex min-h-0 flex-1">
        <Sidebar
          onOpenSettings={() => setPanel('settings')}
          onAdd={() => setPanel('settings')}
        />
        <main className="min-w-0 flex-1 overflow-y-auto">
          {panel === 'settings' ? (
            <SettingsView
              theme={theme}
              onThemeChange={setTheme}
              pollIntervalSeconds={pollInterval}
              onPollIntervalChange={setPollInterval}
              patSuggestionDismissed={patDismissed}
              onDismissPatSuggestion={() => setPatDismissed(true)}
            />
          ) : panel === 'about' ? (
            <AboutView version={APP_VERSION} />
          ) : panel === 'log' ? (
            <CommandLogView />
          ) : selectedSite && selectedServerId ? (
            <SiteView serverId={selectedServerId} site={selectedSite} />
          ) : (
            <EmptyState hasServers={servers.length > 0} />
          )}
        </main>
      </div>
      <footer className="flex shrink-0 items-center justify-end border-t border-border-subtle px-3 py-1">
        <button
          type="button"
          onClick={() => setPanel('log')}
          className="text-xs text-text-tertiary hover:text-text-secondary"
        >
          Command log
        </button>
      </footer>
      <LogDrawer />

      <CommandPalette
        open={paletteOpen}
        onOpenChange={setPaletteOpen}
        items={paletteItems}
      />
    </div>
  );
}

function EmptyState({ hasServers }: { hasServers: boolean }) {
  return (
    <div className="flex h-full items-center justify-center p-6 text-center">
      <p className="max-w-sm text-sm text-text-secondary">
        {hasServers
          ? 'Select a site from the sidebar to get started.'
          : 'Add a server to begin. Popush connects over SSH — no account needed.'}
      </p>
    </div>
  );
}
