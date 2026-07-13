import { create } from 'zustand';
import type { GitStatus, SiteConfig, SiteStatus } from '../types/generated';
import { listSites, getSiteStatus, gitStatus } from '../lib/ipc';

interface SitesState {
  /** Sites keyed by server id. */
  sitesByServer: Record<string, SiteConfig[]>;
  /** Latest known status keyed by site id. */
  statusBySite: Record<string, SiteStatus>;
  /** Latest known git status keyed by site id. */
  gitBySite: Record<string, GitStatus>;
  selectedSiteId: string | null;
  select: (id: string | null) => void;
  refreshSites: (serverId: string) => Promise<void>;
  refreshStatus: (serverId: string, siteId: string) => Promise<void>;
  /** Re-check every known site across all servers. */
  refreshAllStatuses: () => Promise<void>;
  refreshGit: (serverId: string, siteId: string) => Promise<void>;
  setStatus: (siteId: string, status: SiteStatus) => void;
}

export const useSitesStore = create<SitesState>((set, get) => ({
  sitesByServer: {},
  statusBySite: {},
  gitBySite: {},
  selectedSiteId: null,
  select: (id) => set({ selectedSiteId: id }),
  refreshSites: async (serverId) => {
    const sites = await listSites(serverId);
    set((state) => ({
      sitesByServer: { ...state.sitesByServer, [serverId]: sites },
    }));
  },
  refreshStatus: async (serverId, siteId) => {
    let status: SiteStatus | null = null;
    try {
      status = await getSiteStatus(serverId, siteId);
    } catch {
      status = { state: 'stopped' };
    }
    if (status) {
      set((state) => ({
        statusBySite: { ...state.statusBySite, [siteId]: status },
      }));
    }
  },
  refreshAllStatuses: async () => {
    const { sitesByServer, refreshStatus } = get();
    await Promise.all(
      Object.entries(sitesByServer).flatMap(([serverId, sites]) =>
        sites.map((site) => refreshStatus(serverId, site.id)),
      ),
    );
  },
  refreshGit: async (serverId, siteId) => {
    let status: GitStatus | null = null;
    try {
      status = await gitStatus(serverId, siteId);
    } catch {
      status = null;
    }
    if (status) {
      set((state) => ({
        gitBySite: { ...state.gitBySite, [siteId]: status as GitStatus },
      }));
    }
  },
  setStatus: (siteId, status) =>
    set((state) => ({
      statusBySite: { ...state.statusBySite, [siteId]: status },
    })),
}));
