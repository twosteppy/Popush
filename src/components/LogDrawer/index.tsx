// LogDrawer — the bottom drawer (§14.2). Collapsed to a thin bar showing the
// last line cleanly (monospaced, muted, truncated); expandable. Ctrl+` toggles
// it (wired in App). Height is remembered in the pipeline store.
//
// The xterm.js terminal is mounted ONLY when there is real output to show, so
// there are no cursor artifacts or corrupted glyphs in the empty state. Its
// stylesheet is imported, a FitAddon sizes it to the container on open and on
// resize, and its theme is derived from the design tokens (D15).
//
// D14: this is a viewport onto backend log output; it holds no logic. Lines
// come from the pipeline store, which mirrors backend events (§6.3).

import { useEffect, useMemo, useRef } from 'react';
import { ChevronUp, ChevronDown, TerminalSquare } from 'lucide-react';
import 'xterm/css/xterm.css';
import { usePipelineStore } from '../../store/pipeline';

/** Read a design token value from the document root. */
function token(name: string, fallback: string): string {
  if (typeof window === 'undefined') return fallback;
  const v = getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim();
  return v || fallback;
}

interface XtermLike {
  write: (data: string) => void;
  dispose: () => void;
  loadAddon: (addon: unknown) => void;
  open: (el: HTMLElement) => void;
}

export function LogDrawer() {
  const drawerOpen = usePipelineStore((s) => s.drawerOpen);
  const drawerHeight = usePipelineStore((s) => s.drawerHeight);
  const toggleDrawer = usePipelineStore((s) => s.toggleDrawer);
  const steps = usePipelineStore((s) => s.steps);

  // Flatten all streamed output lines from every step, in order.
  const lines = useMemo(() => steps.flatMap((step) => step.output), [steps]);
  const hasOutput = lines.length > 0;
  const lastLine = hasOutput ? lines[lines.length - 1] : null;

  const termRef = useRef<HTMLDivElement | null>(null);
  const instanceRef = useRef<XtermLike | null>(null);
  const fitRef = useRef<{ fit: () => void } | null>(null);
  const writtenRef = useRef(0);

  // Create the terminal only when we are open AND have real output.
  useEffect(() => {
    const shouldMount = drawerOpen && hasOutput && termRef.current;
    if (!shouldMount) return;
    let cancelled = false;

    void (async () => {
      if (instanceRef.current) return;
      try {
        const [{ Terminal }, { FitAddon }] = await Promise.all([
          import('xterm'),
          import('xterm-addon-fit'),
        ]);
        if (cancelled || !termRef.current) return;
        const term = new Terminal({
          fontFamily: token('--font-mono', 'monospace'),
          fontSize: 12,
          lineHeight: 1.35,
          convertEol: true,
          disableStdin: true,
          cursorBlink: false,
          cursorStyle: 'bar',
          theme: {
            background: token('--surface-base', '#0e0e11'),
            foreground: token('--text-primary', '#e8e8ec'),
            cursor: token('--surface-base', '#0e0e11'),
            selectionBackground: token('--accent-muted', '#7c6bf222'),
            black: token('--surface-base', '#0e0e11'),
            brightBlack: token('--text-tertiary', '#6a6a78'),
            red: token('--status-failed', '#f85149'),
            green: token('--status-running', '#3fb950'),
            yellow: token('--status-unknown', '#d29922'),
            blue: token('--accent', '#7c6bf2'),
            magenta: token('--accent', '#7c6bf2'),
            white: token('--text-secondary', '#9a9aa6'),
          },
        }) as unknown as XtermLike;
        const fit = new FitAddon();
        term.loadAddon(fit);
        term.open(termRef.current);
        fit.fit();
        instanceRef.current = term;
        fitRef.current = fit;
        writtenRef.current = 0;
      } catch {
        // xterm unavailable (e.g. under test) — the DOM fallback stays.
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [drawerOpen, hasOutput]);

  // Stream any not-yet-written lines into the terminal.
  useEffect(() => {
    const term = instanceRef.current;
    if (!term) return;
    for (let i = writtenRef.current; i < lines.length; i++) {
      term.write(`${lines[i]}\r\n`);
    }
    writtenRef.current = lines.length;
    fitRef.current?.fit();
  }, [lines]);

  // Refit on container resize.
  useEffect(() => {
    if (!drawerOpen || !hasOutput || !termRef.current) return;
    const el = termRef.current;
    const ro = new ResizeObserver(() => fitRef.current?.fit());
    ro.observe(el);
    return () => ro.disconnect();
  }, [drawerOpen, hasOutput]);

  // Dispose on unmount.
  useEffect(() => {
    return () => {
      instanceRef.current?.dispose();
      instanceRef.current = null;
      fitRef.current = null;
      writtenRef.current = 0;
    };
  }, []);

  return (
    <section
      aria-label="Log output"
      className="flex flex-col border-t border-border-subtle bg-surface-raised"
      style={{ height: drawerOpen ? drawerHeight : 34 }}
    >
      <button
        type="button"
        onClick={toggleDrawer}
        aria-expanded={drawerOpen}
        className="flex h-[34px] shrink-0 items-center gap-2 px-3 text-left text-xs text-text-secondary transition-colors hover:bg-surface-hover"
      >
        {drawerOpen ? (
          <ChevronDown size={14} aria-hidden="true" className="shrink-0" />
        ) : (
          <ChevronUp size={14} aria-hidden="true" className="shrink-0" />
        )}
        <span className="shrink-0 font-medium">Logs</span>
        {lastLine ? (
          <span className="truncate font-mono text-text-tertiary">
            {lastLine}
          </span>
        ) : (
          <span className="text-text-tertiary">No output yet</span>
        )}
      </button>

      {drawerOpen ? (
        hasOutput ? (
          <div
            ref={termRef}
            data-testid="log-terminal"
            className="min-h-0 flex-1 overflow-hidden bg-surface-base px-2 py-1.5"
          />
        ) : (
          <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-2 bg-surface-base text-text-tertiary">
            <TerminalSquare size={20} aria-hidden="true" />
            <p className="text-xs">Waiting for output…</p>
          </div>
        )
      ) : null}
    </section>
  );
}
