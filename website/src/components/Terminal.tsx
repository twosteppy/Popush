import { useEffect, useState } from 'react';
import { Check } from './Icons';

type Row =
  | { kind: 'cmd'; text: string }
  | { kind: 'blank' }
  | { kind: 'step'; name: string; detail: string }
  | { kind: 'done'; text: string };

interface Scene {
  title: string;
  rows: Row[];
}

const step = (name: string, detail: string): Row => ({ kind: 'step', name, detail });
const B: Row = { kind: 'blank' };

const SCENES: Scene[] = [
  {
    title: 'pook-review · ship it',
    rows: [
      { kind: 'cmd', text: 'popush ship pook-review' },
      B,
      step('check', 'server reachable'),
      step('pull', 'fast-forward, 3 files'),
      step('build', 'docker compose build'),
      step('restart', 'docker compose up -d'),
      step('verify', 'pookreview.com  200'),
      B,
      { kind: 'done', text: 'shipped and live in 2m 14s' },
    ],
  },
  {
    title: 'twostep.lol · ship it',
    rows: [
      { kind: 'cmd', text: 'popush ship twostep' },
      B,
      step('check', 'server reachable'),
      step('pull', 'fast-forward, 1 file'),
      step('build', 'vite build'),
      step('sync', 'copied to /srv/twostep'),
      step('verify', 'twostep.lol  200'),
      B,
      { kind: 'done', text: 'shipped and live in 38s' },
    ],
  },
  {
    title: 'uoptimise · ship it',
    rows: [
      { kind: 'cmd', text: 'popush ship uoptimise' },
      B,
      step('check', 'server reachable'),
      step('pull', 'fast-forward, 7 files'),
      step('build', 'docker compose build'),
      step('restart', 'api, portal, admin'),
      step('verify', 'uoptimise.org  200'),
      B,
      { kind: 'done', text: 'shipped and live in 3m 02s' },
    ],
  },
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
  const [scene, setScene] = useState(0);
  const [shown, setShown] = useState(0);
  const rows = SCENES[scene].rows;

  useEffect(() => {
    if (reduced()) {
      setShown(rows.length);
      return;
    }
    const done = shown >= rows.length;
    const cur = rows[shown];
    const delay = done ? 2600 : cur && cur.kind === 'blank' ? 130 : 350;
    const t = window.setTimeout(() => {
      if (done) {
        setScene((s) => (s + 1) % SCENES.length);
        setShown(0);
      } else {
        setShown((s) => s + 1);
      }
    }, delay);
    return () => window.clearTimeout(t);
  }, [shown, scene, rows.length]);

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
          <span className="title">{SCENES[scene].title}</span>
          <span className="live">
            <i /> LIVE
          </span>
        </div>
        {/* Keyed by scene so each deploy types in fresh, no cross-fade. */}
        <div className="term-body" key={scene} aria-hidden="true">
          {rows.map((r, i) => (
            <RowView key={i} row={r} on={i < shown} />
          ))}
          <span className="tcaret" />
        </div>
      </div>
    </div>
  );
}
