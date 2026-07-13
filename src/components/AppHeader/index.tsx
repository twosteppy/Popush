// A slim top bar with the Popush wordmark and a window-drag region
// (data-tauri-drag-region) so the frameless window can be moved. It also hosts
// the command palette hint.

import { Command } from 'lucide-react';
import { Logo } from '../ui/Logo';

interface AppHeaderProps {
  onOpenPalette: () => void;
  /** Return to the home screen (clears the selected site and panel). */
  onHome: () => void;
}

export function AppHeader({ onOpenPalette, onHome }: AppHeaderProps) {
  return (
    <header
      data-tauri-drag-region
      className="flex h-11 shrink-0 select-none items-center gap-3 border-b border-border-strong bg-surface-raised px-3"
    >
      <Logo size={18} onClick={onHome} label="Go to Popush home" />
      <div className="ml-auto">
        <button
          type="button"
          onClick={onOpenPalette}
          className="pressable inline-flex h-7 items-center gap-2 rounded-sm border border-border-strong px-2.5 text-xs text-text-tertiary shadow-hard-sm hover:bg-surface-hover hover:text-text-secondary"
        >
          <Command size={12} aria-hidden="true" />
          Search
          <kbd className="rounded-sm border border-border-strong bg-surface-base px-1 font-mono text-[10px] text-text-tertiary">
            Ctrl K
          </kbd>
        </button>
      </div>
    </header>
  );
}
