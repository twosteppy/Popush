// §6.3: the backend is authoritative. This store holds only a MIRROR of
// backend state so the UI can render without round-tripping. It never mutates
// server-side state; intents go through src/lib/ipc.ts.

import { create } from 'zustand';
import type { ServerConfig } from '../types/generated';
import { listServers, addServer, removeServer } from '../lib/ipc';

interface ServersState {
  servers: ServerConfig[];
  selectedServerId: string | null;
  select: (id: string | null) => void;
  /** Refresh the mirror from the authoritative backend. */
  refresh: () => Promise<void>;
  /**
   * Dispatch the add-server intent to the backend, then re-mirror and select
   * the new server. D14: persistence happens in the backend, not here.
   */
  add: (server: ServerConfig) => Promise<void>;
  /** Dispatch the remove-server intent, then re-mirror. */
  remove: (serverId: string) => Promise<void>;
  /** Replace the mirror (used by event handlers). */
  setServers: (servers: ServerConfig[]) => void;
}

export const useServersStore = create<ServersState>((set, get) => ({
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
  add: async (server) => {
    await addServer(server);
    await get().refresh();
    set({ selectedServerId: server.id });
  },
  remove: async (serverId) => {
    await removeServer(serverId);
    const servers = await listServers();
    set((state) => ({
      servers,
      selectedServerId:
        state.selectedServerId === serverId
          ? (servers[0]?.id ?? null)
          : state.selectedServerId,
    }));
  },
  setServers: (servers) => set({ servers }),
}));
