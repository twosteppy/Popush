const STARS = [
  { top: '-12px', left: '8%', delay: 0 },
  { top: '-16px', left: '38%', delay: 0.25 },
  { top: '-10px', left: '70%', delay: 0.5 },
  { top: '40%', left: '-10px', delay: 0.15 },
  { top: '38%', left: '100%', delay: 0.4 },
  { top: '-6px', left: '92%', delay: 0.65 },
];

/** Little yellow stars that twinkle out when the button is hovered. */
export function StarBurst() {
  return (
    <span className="stars" aria-hidden="true">
      {STARS.map((s, i) => (
        <b
          key={i}
          style={{ top: s.top, left: s.left, animationDelay: `${s.delay}s` }}
        >
          ★
        </b>
      ))}
    </span>
  );
}
