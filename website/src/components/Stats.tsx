import { useEffect, useRef, useState } from 'react';

interface Stat {
  to: number;
  suffix?: string;
  label: string;
  sub: string;
}

const STATS: Stat[] = [
  { to: 1, label: 'Click to ship', sub: 'press Ship It' },
  { to: 6, label: 'Pipeline steps', sub: 'streamed live' },
  { to: 4, label: 'Stack types', sub: 'docker · systemd · pm2 · static' },
  { to: 0, label: 'Bytes uploaded', sub: 'everything stays local' },
];

function useCountUp(to: number, run: boolean) {
  const [n, setN] = useState(0);
  useEffect(() => {
    if (!run) return;
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches || to === 0) {
      setN(to);
      return;
    }
    let raf = 0;
    const start = performance.now();
    const dur = 900;
    const tick = (t: number) => {
      const k = Math.min(1, (t - start) / dur);
      const eased = 1 - Math.pow(1 - k, 3);
      setN(Math.round(eased * to));
      if (k < 1) raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [to, run]);
  return n;
}

function Tile({ stat, run }: { stat: Stat; run: boolean }) {
  const n = useCountUp(stat.to, run);
  return (
    <div className="stat">
      <div className="n">
        {n}
        {stat.suffix ?? ''}
      </div>
      <div className="l">{stat.label}</div>
      <div className="s">{stat.sub}</div>
    </div>
  );
}

export function Stats() {
  const ref = useRef<HTMLDivElement>(null);
  const [run, setRun] = useState(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const io = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) {
          setRun(true);
          io.disconnect();
        }
      },
      { threshold: 0.4 },
    );
    io.observe(el);
    return () => io.disconnect();
  }, []);

  return (
    <section aria-label="At a glance" style={{ paddingTop: 12 }}>
      <div className="wrap">
        <div className="stats reveal" ref={ref}>
          {STATS.map((s) => (
            <Tile key={s.label} stat={s} run={run} />
          ))}
        </div>
      </div>
    </section>
  );
}
