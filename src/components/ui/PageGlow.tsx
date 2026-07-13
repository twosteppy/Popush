// A non-interactive ambient layer: one or two soft, low-opacity radial glows
// tinted with the accent, sitting behind a page's content to give each screen a
// little depth without washing out text.
//
// Purely decorative: pointer-events-none, aria-hidden, and pinned behind
// content (z-0) so it never intercepts clicks or focus.

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
