// §6.3: the backend is authoritative. This store mirrors per-server sites and
// their statuses. Statuses arrive from the backend (polling or events); the UI
// only renders them.

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
  refreshGit: (serverId: string, siteId: string) => Promise<void>;
  setStatus: (siteId: string, status: SiteStatus) => void;
}

export const useSitesStore = create<SitesState>((set) => ({
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
    const status = await getSiteStatus(serverId, siteId);
    if (status) {
      set((state) => ({
        statusBySite: { ...state.statusBySite, [siteId]: status },
      }));
    }
  },
  refreshGit: async (serverId, siteId) => {
    // Outside Tauri (dev/test) or on failure the IPC layer returns null; the
    // GitPanel then falls back to its clean-tree state gracefully.
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
