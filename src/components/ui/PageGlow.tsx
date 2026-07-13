// PageGlow - a non-interactive ambient layer that lays one or two soft,
// low-opacity radial glows tinted with the accent behind a page's content. It
// gives each screen a little depth without washing out text (the tint is capped
// at ~18% of the accent and fades to transparent well before mid-page).
//
// It is purely decorative: pointer-events-none, aria-hidden, and pinned behind
// content (z-0) so it never intercepts clicks or focus. It is static, so it is
// fine under prefers-reduced-motion.
//
// D14: presentation only.

type GlowPlacement = 'top-left' | 'top-right' | 'split' | 'bottom-right';

const GRADIENTS: Record<GlowPlacement, string> = {
  'top-left':
    'radial-gradient(600px circle at 12% 0%, color-mix(in srgb, var(--accent) 18%, transparent), transparent 60%)',
  'top-right':
    'radial-gradient(600px circle at 88% 4%, color-mix(in srgb, var(--accent) 16%, transparent), transparent 60%)',
  split:
    'radial-gradient(560px circle at 15% 0%, color-mix(in srgb, var(--accent) 16%, transparent), transparent 58%), radial-gradient(520px circle at 90% 100%, color-mix(in srgb, var(--accent) 12%, transparent), transparent 60%)',
  'bottom-right':
    'radial-gradient(600px circle at 90% 100%, color-mix(in srgb, var(--accent) 16%, transparent), transparent 60%)',
};

interface PageGlowProps {
  /** Where the glow sits, so different views can vary the ambience. */
  placement?: GlowPlacement;
}

export function PageGlow({ placement = 'split' }: PageGlowProps) {
  return (
    <div
      aria-hidden="true"
      className="pointer-events-none absolute inset-0 z-0"
      style={{ backgroundImage: GRADIENTS[placement] }}
    />
  );
}
