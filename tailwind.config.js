/** @type {import('tailwindcss').Config} */
// Semantic token names map to the CSS custom properties in
// src/styles/tokens.css. Components use these names (e.g. bg-surface-raised) and
// never raw palette classes like bg-gray-800.
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        surface: {
          base: 'var(--surface-base)',
          raised: 'var(--surface-raised)',
          overlay: 'var(--surface-overlay)',
          hover: 'var(--surface-hover)',
        },
        border: {
          subtle: 'var(--border-subtle)',
          strong: 'var(--border-strong)',
        },
        text: {
          primary: 'var(--text-primary)',
          secondary: 'var(--text-secondary)',
          tertiary: 'var(--text-tertiary)',
          inverse: 'var(--text-inverse)',
        },
        accent: {
          DEFAULT: 'var(--accent)',
          hover: 'var(--accent-hover)',
          muted: 'var(--accent-muted)',
        },
        status: {
          running: 'var(--status-running)',
          stopped: 'var(--status-stopped)',
          failed: 'var(--status-failed)',
          unknown: 'var(--status-unknown)',
          working: 'var(--status-working)',
        },
      },
      fontFamily: {
        sans: 'var(--font-sans)',
        mono: 'var(--font-mono)',
      },
      borderRadius: {
        sm: 'var(--radius-sm)',
        md: 'var(--radius-md)',
        lg: 'var(--radius-lg)',
      },
      // 8-bit signature: hard, offset, un-blurred shadows (no soft glow).
      boxShadow: {
        'hard-sm': 'var(--shadow-hard-sm)',
        hard: 'var(--shadow-hard)',
        'hard-accent': 'var(--shadow-hard-accent)',
      },
      borderWidth: {
        DEFAULT: 'var(--border-width)',
        2: 'var(--border-width-strong)',
      },
      spacing: {
        1: 'var(--space-1)',
        2: 'var(--space-2)',
        3: 'var(--space-3)',
        4: 'var(--space-4)',
        6: 'var(--space-6)',
        8: 'var(--space-8)',
      },
      keyframes: {
        // In-progress status dot pulse. Disabled by prefers-reduced-motion via
        // the global rule and the motion-safe variant on the element.
        'status-pulse': {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.5' },
        },
        // Loading skeleton sweep (ui/Skeleton). Gated behind motion-safe at the
        // call site, so prefers-reduced-motion leaves a static block.
        shimmer: {
          '100%': { transform: 'translateX(200%)' },
        },
      },
      animation: {
        'status-pulse': 'status-pulse 1.5s ease-in-out infinite',
        shimmer: 'shimmer 1.6s ease-in-out infinite',
      },
    },
  },
  plugins: [],
};
