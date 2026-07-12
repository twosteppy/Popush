// AppHeader — a slim top bar with the Popush wordmark and a window-drag region
// (data-tauri-drag-region) so the frameless window can be moved. It also hosts
// quick access to the command palette hint.
//
// D14: presentation + navigation intents only.

import { Command } from 'lucide-react';
import { Logo } from '../ui/Logo';

interface AppHeaderProps {
  onOpenPalette: () => void;
}

export function AppHeader({ onOpenPalette }: AppHeaderProps) {
  return (
    <header
      data-tauri-drag-region
      className="flex h-11 shrink-0 select-none items-center gap-3 border-b border-border-subtle bg-surface-raised px-3"
    >
      <Logo size={18} />
      <div className="ml-auto">
        <button
          type="button"
          onClick={onOpenPalette}
          className="inline-flex h-7 items-center gap-2 rounded-md border border-border-subtle px-2.5 text-xs text-text-tertiary transition-colors hover:bg-surface-hover hover:text-text-secondary"
        >
          <Command size={12} aria-hidden="true" />
          Search
          <kbd className="rounded border border-border-strong bg-surface-base px-1 font-mono text-[10px] text-text-tertiary">
            Ctrl K
          </kbd>
        </button>
      </div>
    </header>
  );
}
