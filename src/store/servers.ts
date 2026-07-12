// §6.3: the backend is authoritative. This store holds only a MIRROR of
// backend state so the UI can render without round-tripping. It never mutates
// server-side state; intents go through src/lib/ipc.ts.

import { create } from 'zustand';
import type { ServerConfig } from '../types/generated';
import { listServers } from '../lib/ipc';

interface ServersState {
  servers: ServerConfig[];
  selectedServerId: string | null;
  select: (id: string | null) => void;
  /** Refresh the mirror from the authoritative backend. */
  refresh: () => Promise<void>;
  /** Replace the mirror (used by event handlers). */
  setServers: (servers: ServerConfig[]) => void;
}

export const useServersStore = create<ServersState>((set) => ({
  servers: [],
  selectedServerId: null,
  select: (id) => set({ selectedServerId: id }),
  refresh: async () => {
    const servers = await listServers();
    set((state) => ({
      servers,
      selectedServerId: state.selectedServerId ?? servers[0]?.id ?? null,
    }));
  },
  setServers: (servers) => set({ servers }),
}));
