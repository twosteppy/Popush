// Fix 1: the modal registry enforces one modal at a time. The Ctrl+K palette
// handler consults nextPaletteOpen(current, anyModalOpen); this pins down its
// truth table so the palette can never stack on top of another dialog.

import { beforeEach, describe, expect, it } from 'vitest';
import { isAnyModalOpen, nextPaletteOpen, useModalStore } from '../modals';

describe('modal registry', () => {
  beforeEach(() => {
    useModalStore.setState({ openCount: 0 });
  });

  it('tracks open modals via register/unregister', () => {
    expect(isAnyModalOpen()).toBe(false);
    useModalStore.getState().register();
    expect(isAnyModalOpen()).toBe(true);
    useModalStore.getState().unregister();
    expect(isAnyModalOpen()).toBe(false);
  });

  it('never lets openCount go negative', () => {
    useModalStore.getState().unregister();
    expect(useModalStore.getState().openCount).toBe(0);
  });
});

describe('nextPaletteOpen (Ctrl+K guard)', () => {
  it('opens the palette when nothing else is open', () => {
    expect(nextPaletteOpen(false, false)).toBe(true);
  });

  it('is a no-op (stays closed) while another dialog is open', () => {
    expect(nextPaletteOpen(false, true)).toBe(false);
  });

  it('always closes the palette when it is the open modal', () => {
    expect(nextPaletteOpen(true, false)).toBe(false);
    expect(nextPaletteOpen(true, true)).toBe(false);
  });
});
