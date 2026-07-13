// A slim top bar with the Popush wordmark and a window-drag region
// (data-tauri-drag-region) so the frameless window can be moved. It also hosts
// the command palette hint.

import { Search } from 'lucide-react';
import { Logo } from '../ui/Logo';

interface AppHeaderProps {
  onOpenPalette: () => void;
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
          aria-label="Search sites and actions"
          className="pressable inline-flex h-7 items-center gap-2 rounded-sm border border-border-strong bg-surface-base px-2.5 text-xs text-text-secondary shadow-hard-sm hover:border-accent hover:bg-surface-hover hover:text-text-primary"
        >
          <Search size={13} aria-hidden="true" />
          <span>Search</span>
          <kbd className="ml-1 rounded-sm border border-border-subtle bg-surface-raised px-1.5 py-0.5 font-mono text-[10px] text-text-tertiary">
            Ctrl K
          </kbd>
        </button>
      </div>
    </header>
  );
}
