// Tracks whether a modal is currently open. Every dialog built on the shared
// ui/Dialog wrapper increments a counter while it is mounted-open and
// decrements when it closes. Global shortcuts (Ctrl+K) consult this so only one
// modal is open at a time and the command palette never stacks on another
// dialog.

import { create } from 'zustand';

interface ModalState {
  /** Number of modals currently open. */
  openCount: number;
  /** Register a modal as open. */
  register: () => void;
  /** Unregister a modal (on close or unmount). */
  unregister: () => void;
}

export const useModalStore = create<ModalState>((set) => ({
  openCount: 0,
  register: () => set((s) => ({ openCount: s.openCount + 1 })),
  unregister: () => set((s) => ({ openCount: Math.max(0, s.openCount - 1) })),
}));

/** True when at least one modal (dialog or the command palette) is open. */
export function isAnyModalOpen(): boolean {
  return useModalStore.getState().openCount > 0;
}

/**
 * Decide the next command-palette open state for a Ctrl+K press (Fix 1). The
 * palette may always close itself, but it must never open while another modal
 * is already open, so only one modal is visible at a time.
 */
export function nextPaletteOpen(
  current: boolean,
  anyModalOpen: boolean,
): boolean {
  if (current) return false; // the palette is open; Ctrl+K closes it
  if (anyModalOpen) return false; // another dialog is open; stay closed
  return true; // nothing open; open the palette
}
