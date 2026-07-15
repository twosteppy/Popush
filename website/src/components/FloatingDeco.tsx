interface Flower {
  c: string;
  top: string;
  left: string;
  size: number;
  delay: number;
  dur: number;
}

const FLOWERS: Flower[] = [
  { c: '✿', top: '11%', left: '5%', size: 26, delay: 0, dur: 13 },
  { c: '✦', top: '22%', left: '90%', size: 18, delay: 2, dur: 11 },
  { c: '❀', top: '58%', left: '8%', size: 22, delay: 1, dur: 15 },
  { c: '✿', top: '74%', left: '85%', size: 20, delay: 3, dur: 12 },
  { c: '✦', top: '42%', left: '54%', size: 14, delay: 1.5, dur: 16 },
  { c: '❁', top: '88%', left: '38%', size: 18, delay: 2.5, dur: 14 },
  { c: '✿', top: '33%', left: '28%', size: 15, delay: 3.5, dur: 13 },
];

/** A few slow-drifting pixel-pink flowers behind everything, purely decorative. */
export function FloatingDeco() {
  return (
    <div className="deco" aria-hidden="true">
      {FLOWERS.map((f, i) => (
        <b
          key={i}
          style={{
            top: f.top,
            left: f.left,
            fontSize: f.size,
            animationDelay: `${f.delay}s`,
            animationDuration: `${f.dur}s`,
          }}
        >
          {f.c}
        </b>
      ))}
    </div>
  );
}
