import type { SVGProps } from 'react';

type IconProps = SVGProps<SVGSVGElement>;

const stroke = (props: IconProps): IconProps => ({
  viewBox: '0 0 24 24',
  fill: 'none',
  stroke: 'currentColor',
  strokeWidth: 2,
  strokeLinecap: 'round',
  strokeLinejoin: 'round',
  'aria-hidden': true,
  ...props,
});

export const Check = (p: IconProps) => (
  <svg {...stroke({ strokeWidth: 2.5, ...p })}>
    <path d="M20 6 9 17l-5-5" />
  </svg>
);

export const Dash = (p: IconProps) => (
  <svg {...stroke({ strokeWidth: 2.5, ...p })}>
    <path d="M5 12h14" />
  </svg>
);

export const Circle = (p: IconProps) => (
  <svg {...stroke(p)}>
    <circle cx="12" cy="12" r="8" />
  </svg>
);

export const Spinner = (p: IconProps) => (
  <svg {...stroke({ strokeWidth: 2.5, ...p })}>
    <path d="M21 12a9 9 0 1 1-6.2-8.6" />
  </svg>
);

export const Download = (p: IconProps) => (
  <svg {...stroke(p)}>
    <path d="M12 3v12m0 0 4-4m-4 4-4-4" />
    <path d="M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2" />
  </svg>
);

export const Rocket = (p: IconProps) => (
  <svg {...stroke(p)}>
    <path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z" />
    <path d="m12 15-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z" />
    <path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0" />
  </svg>
);

export const ArrowUpRight = (p: IconProps) => (
  <svg {...stroke(p)}>
    <path d="M7 17 17 7M7 7h10v10" />
  </svg>
);

export const Clock = (p: IconProps) => (
  <svg {...stroke(p)}>
    <circle cx="12" cy="12" r="9" />
    <path d="M12 7v5l3 2" />
  </svg>
);

export const Stack = (p: IconProps) => (
  <svg {...stroke(p)}>
    <rect x="3" y="3" width="18" height="18" rx="1" />
    <path d="M3 9h18M9 21V9" />
  </svg>
);

export const Lock = (p: IconProps) => (
  <svg {...stroke(p)}>
    <rect x="4" y="10" width="16" height="11" rx="2" />
    <path d="M8 10V7a4 4 0 0 1 8 0v3" />
  </svg>
);

export const Sun = (p: IconProps) => (
  <svg {...stroke(p)}>
    <circle cx="12" cy="12" r="4" />
    <path d="M12 2v2m0 16v2M4.9 4.9l1.4 1.4m11.4 11.4 1.4 1.4M2 12h2m16 0h2M4.9 19.1l1.4-1.4m11.4-11.4 1.4-1.4" />
  </svg>
);

export const Moon = (p: IconProps) => (
  <svg {...stroke(p)}>
    <path d="M21 12.8A9 9 0 1 1 11.2 3 7 7 0 0 0 21 12.8z" />
  </svg>
);

export const ChevronDown = (p: IconProps) => (
  <svg {...stroke(p)}>
    <path d="m6 9 6 6 6-6" />
  </svg>
);

export const Code = (p: IconProps) => (
  <svg {...stroke(p)}>
    <path d="m16 18 6-6-6-6M8 6l-6 6 6 6" />
  </svg>
);

export const GitHub = (p: IconProps) => (
  <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden {...p}>
    <path d="M12 2A10 10 0 0 0 2 12c0 4.42 2.87 8.17 6.84 9.5.5.08.66-.22.66-.48v-1.7c-2.78.6-3.37-1.34-3.37-1.34-.45-1.16-1.11-1.47-1.11-1.47-.9-.62.07-.6.07-.6 1 .07 1.53 1.03 1.53 1.03.9 1.52 2.34 1.08 2.91.83.09-.65.35-1.09.63-1.34-2.22-.25-4.55-1.11-4.55-4.94 0-1.09.39-1.98 1.03-2.68-.1-.25-.45-1.27.1-2.65 0 0 .84-.27 2.75 1.02a9.6 9.6 0 0 1 5 0c1.9-1.29 2.74-1.02 2.74-1.02.55 1.38.2 2.4.1 2.65.64.7 1.03 1.59 1.03 2.68 0 3.84-2.34 4.68-4.57 4.93.36.31.68.92.68 1.85v2.74c0 .27.16.57.67.48A10 10 0 0 0 22 12 10 10 0 0 0 12 2z" />
  </svg>
);
