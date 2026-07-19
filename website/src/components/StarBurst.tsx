const STARS = [
  { top: '-14px', left: '6%', delay: 0, size: 13, ch: '★' },
  { top: '-18px', left: '30%', delay: 0.2, size: 10, ch: '✦' },
  { top: '-12px', left: '52%', delay: 0.45, size: 15, ch: '★' },
  { top: '-16px', left: '78%', delay: 0.3, size: 11, ch: '✦' },
  { top: '-7px', left: '94%', delay: 0.6, size: 12, ch: '★' },
  { top: '34%', left: '-12px', delay: 0.15, size: 10, ch: '✦' },
  { top: '46%', left: '101%', delay: 0.52, size: 13, ch: '★' },
  { top: '82%', left: '18%', delay: 0.7, size: 9, ch: '✦' },
];

/** Little yellow stars that twinkle out when the button is hovered. */
export function StarBurst() {
  return (
    <span className="stars" aria-hidden="true">
      {STARS.map((s, i) => (
        <b
          key={i}
          style={{
            top: s.top,
            left: s.left,
            fontSize: `${s.size}px`,
            animationDelay: `${s.delay}s`,
          }}
        >
          {s.ch}
        </b>
      ))}
    </span>
  );
}
