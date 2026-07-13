import { create } from 'zustand';
import type { ServerConfig } from '../types/generated';
import { listServers, addServer, removeServer } from '../lib/ipc';

interface ServersState {
  servers: ServerConfig[];
  selectedServerId: string | null;
  /** True while the initial list_servers mirror is still resolving. */
  loading: boolean;
  /** True once the mirror has been hydrated at least once. */
  hydrated: boolean;
  select: (id: string | null) => void;
  /** Refresh the mirror from the authoritative backend. */
  refresh: () => Promise<void>;
  /**
   * Dispatch the add-server intent to the backend, then re-mirror and select
   * the new server.
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
  loading: false,
  hydrated: false,
  select: (id) => set({ selectedServerId: id }),
  refresh: async () => {
    set((state) => ({ loading: !state.hydrated }));
    try {
      const servers = await listServers();
      set((state) => ({
        servers,
        selectedServerId: state.selectedServerId ?? servers[0]?.id ?? null,
      }));
    } finally {
      set({ loading: false, hydrated: true });
    }
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
