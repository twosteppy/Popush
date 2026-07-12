// §6.3: the backend is authoritative. The pipeline runs entirely in the
// backend; this store mirrors the PipelineState it emits via events. The UI
// renders these steps and never decides step order or semantics (D14).

import { create } from 'zustand';
import type { PipelineState } from '../types/generated';

interface PipelineStore {
  /** The id of the pipeline currently being mirrored, if any. */
  pipelineId: string | null;
  /** The latest PipelineState snapshot from the backend. */
  state: PipelineState | null;
  /** True while the drawer/log height is being remembered. */
  drawerHeight: number;
  drawerOpen: boolean;
  begin: (pipelineId: string) => void;
  /** Apply an authoritative snapshot pushed by the backend. */
  update: (state: PipelineState) => void;
  reset: () => void;
  setDrawerHeight: (height: number) => void;
  toggleDrawer: () => void;
  setDrawerOpen: (open: boolean) => void;
}

export const usePipelineStore = create<PipelineStore>((set) => ({
  pipelineId: null,
  state: null,
  drawerHeight: 220,
  drawerOpen: false,
  begin: (pipelineId) => set({ pipelineId, state: null }),
  update: (state) => set({ state }),
  reset: () => set({ pipelineId: null, state: null }),
  setDrawerHeight: (height) => set({ drawerHeight: height }),
  toggleDrawer: () => set((s) => ({ drawerOpen: !s.drawerOpen })),
  setDrawerOpen: (open) => set({ drawerOpen: open }),
}));
