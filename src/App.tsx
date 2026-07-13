import { useEffect, useMemo, useState } from 'react';
import { AnimatePresence, motion, useReducedMotion } from 'framer-motion';
import { AppHeader } from './components/AppHeader';
import { Sidebar } from './components/Sidebar';
import { LogDrawer } from './components/LogDrawer';
import { EmptyState } from './components/EmptyState';
import { AddServerDialog } from './components/AddServerDialog';
import { AddSiteDialog } from './components/AddSiteDialog';
import { CommandPalette, type PaletteItem } from './components/CommandPalette';
import { PageGlow } from './components/ui/PageGlow';
import { SiteView } from './views/SiteView';
import { SettingsView } from './views/SettingsView';
import { AboutView } from './views/AboutView';
import { HelpView } from './views/HelpView';
import { CommandLogView } from './views/CommandLogView';
import { WizardContainer } from './views/WizardContainer';
import { useServersStore } from './store/servers';
import { useSitesStore } from './store/sites';
import { usePipelineStore } from './store/pipeline';
import { usePipelineEvents } from './hooks/usePipelineEvents';
import { isAnyModalOpen, nextPaletteOpen } from './store/modals';
import type { Theme } from './types/generated';

export type Panel = 'site' | 'settings' | 'about' | 'help' | 'log' | 'wizard';

const APP_VERSION = '0.1.0';

export function App() {
  const [panel, setPanel] = useState<Panel>('site');
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [addServerOpen, setAddServerOpen] = useState(false);
  const [addSiteOpen, setAddSiteOpen] = useState(false);
  const [theme, setTheme] = useState<Theme>('system');
  const [pollInterval, setPollInterval] = useState(30);
  const [patDismissed, setPatDismissed] = useState(false);
  const reduce = useReducedMotion();

  const { servers, selectedServerId, refresh } = useServersStore();
  const {
    sitesByServer,
    selectedSiteId,
    refreshSites,
    select: selectSite,
  } = useSitesStore();
  const { toggleDrawer } = usePipelineStore();

  usePipelineEvents();

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    if (selectedServerId) void refreshSites(selectedServerId);
  }, [selectedServerId, refreshSites]);

  useEffect(() => {
    const el = document.documentElement;
    if (theme === 'system') el.removeAttribute('data-theme');
    else el.setAttribute('data-theme', theme);
  }, [theme]);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.ctrlKey && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setPaletteOpen((open) => nextPaletteOpen(open, isAnyModalOpen()));
      } else if (e.ctrlKey && e.key === '`') {
        e.preventDefault();
        toggleDrawer();
      }
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [toggleDrawer]);

  const openPalette = () => {
    if (!isAnyModalOpen()) setPaletteOpen(true);
  };

  const goHome = () => {
    selectSite(null);
    setPanel('site');
  };

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
      id: 'action:add-server',
      label: 'Add a server',
      kind: 'Action',
      onSelect: () => setAddServerOpen(true),
    });
    items.push({
      id: 'action:wizard',
      label: 'Run the setup wizard',
      kind: 'Action',
      onSelect: () => setPanel('wizard'),
    });
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
      id: 'action:help',
      label: 'What is Popush?',
      kind: 'Action',
      onSelect: () => setPanel('help'),
    });
    items.push({
      id: 'action:about',
      label: 'About Popush',
      kind: 'Action',
      onSelect: () => setPanel('about'),
    });
    return items;
  }, [sitesByServer, selectSite]);

  const content =
    panel === 'settings' ? (
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
    ) : panel === 'help' ? (
      <HelpView onAddServer={() => setAddServerOpen(true)} />
    ) : panel === 'log' ? (
      <CommandLogView />
    ) : panel === 'wizard' ? (
      <WizardContainer serverId={selectedServerId} siteId={selectedSiteId} />
    ) : selectedSite && selectedServerId ? (
      <SiteView serverId={selectedServerId} site={selectedSite} />
    ) : (
      <EmptyState
        hasServers={servers.length > 0}
        onAddServer={() => setAddServerOpen(true)}
        onRunWizard={() => setPanel('wizard')}
        onOpenHelp={() => setPanel('help')}
      />
    );

  const contentKey = `${panel}:${selectedSite?.id ?? 'none'}`;

  const glowPlacement =
    panel === 'settings' || panel === 'about'
      ? 'top-right'
      : panel === 'help'
        ? 'split'
        : panel === 'log'
          ? 'bottom-right'
          : 'top-left';

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden bg-surface-base text-text-primary">
      <AppHeader onOpenPalette={openPalette} onHome={goHome} />
      <div className="flex min-h-0 flex-1">
        <Sidebar
          activePanel={panel}
          onOpenSettings={() => setPanel('settings')}
          onOpenWizard={() => setPanel('wizard')}
          onOpenHelp={() => setPanel('help')}
          onAddServer={() => setAddServerOpen(true)}
          onAddSite={() => setAddSiteOpen(true)}
          onSelectSite={() => setPanel('site')}
        />
        {/* The ambient glow sits on the main viewport behind a nested scroll
         * region, so it never scrolls and can't reveal a hard bottom edge on
         * tall views. The scroll container is an absolute inset-0 child with a
         * definite height, so the inner view keeps its h-full centring while
         * long content scrolls over the static glow. */}
        <main className="relative min-w-0 flex-1 overflow-hidden">
          <PageGlow placement={glowPlacement} />
          <div className="absolute inset-0 z-10 overflow-y-auto">
            <AnimatePresence mode="wait" initial={false}>
              <motion.div
                key={contentKey}
                initial={reduce ? false : { opacity: 0, y: 6 }}
                animate={{ opacity: 1, y: 0 }}
                exit={reduce ? undefined : { opacity: 0, y: -4 }}
                transition={{ duration: 0.16, ease: 'easeOut' }}
                className="h-full"
              >
                {content}
              </motion.div>
            </AnimatePresence>
          </div>
        </main>
      </div>

      <LogDrawer />

      <AddServerDialog open={addServerOpen} onOpenChange={setAddServerOpen} />

      <AddSiteDialog
        open={addSiteOpen}
        onOpenChange={setAddSiteOpen}
        serverId={selectedServerId}
      />

      <CommandPalette
        open={paletteOpen}
        onOpenChange={setPaletteOpen}
        items={paletteItems}
      />
    </div>
  );
}
