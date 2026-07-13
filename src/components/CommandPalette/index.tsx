// CommandPalette - Ctrl+K fuzzy finder over sites and actions, built on the
// Radix Dialog wrapper (§20). A simple substring filter is enough.
//
// D14: it emits a chosen intent (select a site, run an action) upward; it does
// not perform the action.

import { useMemo, useState } from 'react';
import { Dialog } from '../ui/Dialog';

export interface PaletteItem {
  id: string;
  label: string;
  /** Grouping hint, e.g. "Site" or "Action". */
  kind: string;
  onSelect: () => void;
}

interface CommandPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  items: PaletteItem[];
}

export function CommandPalette({
  open,
  onOpenChange,
  items,
}: CommandPaletteProps) {
  const [query, setQuery] = useState('');

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return items;
    return items.filter(
      (item) =>
        item.label.toLowerCase().includes(q) ||
        item.kind.toLowerCase().includes(q),
    );
  }, [items, query]);

  function choose(item: PaletteItem) {
    item.onSelect();
    onOpenChange(false);
    setQuery('');
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        onOpenChange(next);
        if (!next) setQuery('');
      }}
      title="Command palette"
      hideTitle
    >
      <input
        autoFocus
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="Search sites and actions…"
        className="mb-3 w-full rounded-sm border border-border-strong bg-surface-base px-3 py-2 text-sm text-text-primary placeholder:text-text-tertiary focus:border-accent focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
      />
      <ul className="max-h-72 overflow-y-auto">
        {filtered.length === 0 ? (
          <li className="px-2 py-2 text-sm text-text-tertiary">No matches.</li>
        ) : (
          filtered.map((item) => (
            <li key={item.id}>
              <button
                type="button"
                onClick={() => choose(item)}
                className="flex w-full items-center justify-between rounded-sm border border-transparent px-2 py-2 text-left text-sm text-text-primary hover:border-border-subtle hover:bg-surface-hover"
              >
                <span>{item.label}</span>
                <span className="label-mono text-[10px] text-text-tertiary">
                  {item.kind}
                </span>
              </button>
            </li>
          ))
        )}
      </ul>
    </Dialog>
  );
}
