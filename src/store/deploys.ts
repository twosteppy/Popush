import { create } from 'zustand';

/** Remembers when each site was last pushed, persisted on this machine so the
 * label survives an app restart. Keyed by site id, stored as ISO timestamps. */
const KEY = 'popush.lastPushed';

function load(): Record<string, string> {
  try {
    const raw = localStorage.getItem(KEY);
    const parsed = raw ? (JSON.parse(raw) as unknown) : null;
    return parsed && typeof parsed === 'object'
      ? (parsed as Record<string, string>)
      : {};
  } catch {
    return {};
  }
}

function save(map: Record<string, string>): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(map));
  } catch {
    /* ignore: a full or disabled store just means no persistence */
  }
}

interface DeployStore {
  lastPushedBySite: Record<string, string>;
  /** Record a push for a site (defaults to now). */
  recordPush: (siteId: string, iso?: string) => void;
}

export const useDeployStore = create<DeployStore>((set) => ({
  lastPushedBySite: load(),
  recordPush: (siteId, iso) =>
    set((s) => {
      const next = {
        ...s.lastPushedBySite,
        [siteId]: iso ?? new Date().toISOString(),
      };
      save(next);
      return { lastPushedBySite: next };
    }),
}));
