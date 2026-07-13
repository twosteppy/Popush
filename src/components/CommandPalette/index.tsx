// Ctrl+K fuzzy finder over sites and actions, built on the Dialog wrapper. A
// simple substring filter is enough. It emits the chosen intent upward and does
// not perform the action itself.

import { useEffect, useMemo, useRef, useState } from 'react';
import { Dialog } from '../ui/Dialog';
import { cn } from '../../lib/cn';

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
  const [active, setActive] = useState(0);
  const listRef = useRef<HTMLUListElement | null>(null);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return items;
    return items.filter(
      (item) =>
        item.label.toLowerCase().includes(q) ||
        item.kind.toLowerCase().includes(q),
    );
  }, [items, query]);

  // Keep the highlighted row in range as the list narrows, and scroll it into
  // view when the keyboard moves it.
  useEffect(() => {
    setActive(0);
  }, [query]);

  useEffect(() => {
    const el = listRef.current?.querySelector<HTMLElement>(
      '[data-active="true"]',
    );
    el?.scrollIntoView({ block: 'nearest' });
  }, [active, filtered.length]);

  function choose(item: PaletteItem) {
    item.onSelect();
    onOpenChange(false);
    setQuery('');
  }

  function onKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (filtered.length === 0) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setActive((i) => (i + 1) % filtered.length);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setActive((i) => (i - 1 + filtered.length) % filtered.length);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const item = filtered[active];
      if (item) choose(item);
    }
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
        onKeyDown={onKeyDown}
        placeholder="Search sites and actions…"
        className="mb-3 w-full rounded-sm border border-border-strong bg-surface-base px-3 py-2 text-sm text-text-primary transition-colors placeholder:text-text-tertiary focus:border-accent focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
      />
      <ul ref={listRef} className="max-h-72 overflow-y-auto">
        {filtered.length === 0 ? (
          <li className="px-2 py-2 text-sm text-text-tertiary">No matches.</li>
        ) : (
          filtered.map((item, index) => {
            const isActive = index === active;
            return (
              <li key={item.id}>
                <button
                  type="button"
                  onClick={() => choose(item)}
                  onMouseMove={() => setActive(index)}
                  data-active={isActive || undefined}
                  className={cn(
                    'flex w-full items-center justify-between rounded-sm border px-2 py-2 text-left text-sm transition-colors',
                    isActive
                      ? 'border-border-subtle bg-surface-hover text-text-primary'
                      : 'border-transparent text-text-primary',
                  )}
                >
                  <span>{item.label}</span>
                  <span
                    className={cn(
                      'label-mono text-[10px]',
                      isActive ? 'text-accent' : 'text-text-tertiary',
                    )}
                  >
                    {item.kind}
                  </span>
                </button>
              </li>
            );
          })
        )}
      </ul>
    </Dialog>
  );
}
