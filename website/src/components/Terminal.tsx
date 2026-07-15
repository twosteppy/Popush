import { useEffect, useRef, useState } from 'react';

type Seg = { t: string; c?: string };
type Line = Seg[];

const pad = (s: string, n: number) => s + ' '.repeat(Math.max(0, n - s.length));

function step(name: string, detail: string): Line {
  return [
    { t: '  ' },
    { t: pad(name, 9), c: 'k' },
    { t: pad(detail, 24), c: 'm' },
    { t: 'ok', c: 's' },
  ];
}

const LINES: Line[] = [
  [
    { t: '$ ', c: 'k' },
    { t: 'popush ship pook-review', c: 'cmd-t' },
  ],
  [{ t: ' ' }],
  step('check', 'server reachable'),
  step('pull', 'fast-forward, 3 files'),
  step('build', 'docker compose build'),
  step('restart', 'docker compose up -d'),
  step('verify', 'pookreview.com  200'),
  [{ t: ' ' }],
  [
    { t: '  ' },
    { t: '♥ ', c: 'k' },
    { t: 'shipped and live', c: 's' },
    { t: '  in 2m 14s', c: 'm' },
  ],
];

const lineLen = (l: Line) => l.reduce((n, s) => n + s.t.length, 0);

/** Render the first `upto` characters of a line, keeping per-segment colour. */
function renderLine(line: Line, upto: number) {
  let left = upto;
  const out: React.ReactNode[] = [];
  line.forEach((seg, i) => {
    if (left <= 0) return;
    const text = seg.t.slice(0, left);
    left -= seg.t.length;
    out.push(
      seg.c ? (
        <span key={i} className={seg.c}>
          {text}
        </span>
      ) : (
        <span key={i}>{text}</span>
      ),
    );
  });
  return out;
}

const prefersReduced = () =>
  typeof window !== 'undefined' &&
  window.matchMedia('(prefers-reduced-motion: reduce)').matches;

export function Terminal() {
  const [li, setLi] = useState(0); // current line
  const [ch, setCh] = useState(0); // chars typed on current line
  const timer = useRef<number | undefined>(undefined);

  useEffect(() => {
    if (prefersReduced()) {
      setLi(LINES.length);
      return;
    }
    const done = li >= LINES.length;
    const full = !done && ch >= lineLen(LINES[li]);

    let delay = 16; // per character
    if (done) delay = 2600; // hold the finished frame, then loop
    else if (full) delay = LINES[li].length <= 1 ? 90 : 280; // pause at line end

    timer.current = window.setTimeout(() => {
      if (done) {
        setLi(0);
        setCh(0);
      } else if (full) {
        setLi((n) => n + 1);
        setCh(0);
      } else {
        setCh((n) => n + 1);
      }
    }, delay);

    return () => window.clearTimeout(timer.current);
  }, [li, ch]);

  const finished = li >= LINES.length;

  return (
    <div className="term" role="img" aria-label="Popush deploying a site: check, pull, build, restart, verify, then shipped and live.">
      <div className="term-bar">
        <div className="dots">
          <i />
          <i />
          <i />
        </div>
        <span className="title">popush ship it</span>
        <span className="live">
          <i /> LIVE
        </span>
      </div>
      <div className="term-body" aria-hidden="true">
        {LINES.map((line, i) => {
          if (i > li) return <span key={i} className="tline">{' '}</span>;
          const upto = i < li || finished ? lineLen(line) : ch;
          const isCursor = !finished && i === li;
          return (
            <span key={i} className="tline">
              {renderLine(line, upto)}
              {isCursor ? <span className="tcursor" /> : null}
            </span>
          );
        })}
        {finished ? <span className="tcursor" /> : null}
      </div>
    </div>
  );
}
