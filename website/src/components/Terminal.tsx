import { useEffect, useState } from 'react';
import { Check } from './Icons';

type Row =
  | { kind: 'cmd'; text: string }
  | { kind: 'blank' }
  | { kind: 'step'; name: string; detail: string }
  | { kind: 'done'; text: string };

const ROWS: Row[] = [
  { kind: 'cmd', text: 'popush ship pook-review' },
  { kind: 'blank' },
  { kind: 'step', name: 'check', detail: 'server reachable' },
  { kind: 'step', name: 'pull', detail: 'fast-forward, 3 files' },
  { kind: 'step', name: 'build', detail: 'docker compose build' },
  { kind: 'step', name: 'restart', detail: 'docker compose up -d' },
  { kind: 'step', name: 'verify', detail: 'pookreview.com  200' },
  { kind: 'blank' },
  { kind: 'done', text: 'shipped and live in 2m 14s' },
];

const reduced = () =>
  typeof window !== 'undefined' &&
  window.matchMedia('(prefers-reduced-motion: reduce)').matches;

function RowView({ row, on }: { row: Row; on: boolean }) {
  const cls = `trow ${row.kind}${on ? ' in' : ''}`;
  if (row.kind === 'blank') return <div className={cls} />;
  if (row.kind === 'cmd') {
    return (
      <div className={cls}>
        <span className="prompt">$</span>
        <span className="cmd-t">{row.text}</span>
      </div>
    );
  }
  if (row.kind === 'done') {
    return (
      <div className={cls}>
        <span className="heart">♥</span>
        <span className="dn">{row.text}</span>
      </div>
    );
  }
  return (
    <div className={cls}>
      <span className="ok">
        <Check strokeWidth={3} />
      </span>
      <span className="nm">{row.name}</span>
      <span className="dt">{row.detail}</span>
    </div>
  );
}

export function Terminal() {
  const [shown, setShown] = useState(0);

  useEffect(() => {
    if (reduced()) {
      setShown(ROWS.length);
      return;
    }
    const done = shown >= ROWS.length;
    const cur = ROWS[shown];
    const delay = done ? 2800 : cur && cur.kind === 'blank' ? 130 : 350;
    const t = window.setTimeout(() => setShown((s) => (done ? 0 : s + 1)), delay);
    return () => window.clearTimeout(t);
  }, [shown]);

  return (
    <div className="term-wrap">
      <div className="term-glow" aria-hidden="true" />
      <div
        className="term"
        role="img"
        aria-label="Popush deploying a site: check, pull, build, restart, verify, then shipped and live."
      >
      <div className="term-bar">
        <div className="dots">
          <i />
          <i />
          <i />
        </div>
        <span className="title">pook-review · ship it</span>
        <span className="live">
          <i /> LIVE
        </span>
      </div>
      <div className="term-body" aria-hidden="true">
        {ROWS.map((r, i) => (
          <RowView key={i} row={r} on={i < shown} />
        ))}
        <span className="tcaret" />
      </div>
      </div>
    </div>
  );
}
