// LogDrawer — the bottom drawer (§14.2). Collapsed to a thin bar showing the
// last line; expandable to half/full. Ctrl+` toggles it (wired in App). Height
// is remembered in the pipeline store.
//
// The xterm.js terminal is lazy-created only inside Tauri / a real DOM with a
// measured container, so tests and the plain dev server never need a terminal.
//
// D14: this is a viewport onto backend log output; it holds no logic.

import { useEffect, useRef } from 'react';
import { ChevronUp, ChevronDown } from 'lucide-react';
import { usePipelineStore } from '../../store/pipeline';

interface LogDrawerProps {
  /** The last log line, shown on the collapsed bar. */
  lastLine?: string;
}

export function LogDrawer({ lastLine }: LogDrawerProps) {
  const { drawerOpen, drawerHeight, toggleDrawer } = usePipelineStore();
  const termRef = useRef<HTMLDivElement | null>(null);
  const instanceRef = useRef<{ dispose: () => void } | null>(null);

  useEffect(() => {
    if (!drawerOpen || !termRef.current) return;
    let cancelled = false;

    // Lazy-create xterm only when the drawer is actually open and mounted.
    // Guarded so tests (jsdom) don't need a real terminal backend.
    void (async () => {
      if (instanceRef.current) return;
      try {
        const { Terminal } = await import('xterm');
        if (cancelled || !termRef.current) return;
        const term = new Terminal({
          fontFamily: 'var(--font-mono)',
          fontSize: 12,
          convertEol: true,
          disableStdin: true,
        });
        term.open(termRef.current);
        instanceRef.current = term;
      } catch {
        // xterm unavailable (e.g. under test) — fall back to plain text.
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [drawerOpen]);

  useEffect(() => {
    return () => {
      instanceRef.current?.dispose();
      instanceRef.current = null;
    };
  }, []);

  return (
    <section
      aria-label="Log output"
      className="flex flex-col border-t border-border-subtle bg-surface-raised"
      style={{ height: drawerOpen ? drawerHeight : 32 }}
    >
      <button
        type="button"
        onClick={toggleDrawer}
        aria-expanded={drawerOpen}
        className="flex h-8 shrink-0 items-center gap-2 px-3 text-left text-xs text-text-secondary hover:bg-surface-hover"
      >
        {drawerOpen ? (
          <ChevronDown size={14} aria-hidden="true" />
        ) : (
          <ChevronUp size={14} aria-hidden="true" />
        )}
        <span className="font-mono truncate">{lastLine ?? 'Logs'}</span>
      </button>

      {drawerOpen ? (
        <div
          ref={termRef}
          className="min-h-0 flex-1 overflow-auto bg-surface-base p-2 font-mono text-xs text-text-secondary"
        >
          {/* Fallback content when no terminal is attached. */}
          {instanceRef.current ? null : (
            <span className="text-text-tertiary">Waiting for log output…</span>
          )}
        </div>
      ) : null}
    </section>
  );
}
