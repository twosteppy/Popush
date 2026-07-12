# Bundled fonts (D16)

Popush bundles its fonts. They are **never** loaded from a CDN.

Vendor the following `.woff2` files into this directory. They are compiled
into the app bundle at build time and referenced by the local `@font-face`
rules in `../globals.css`:

- `Inter-Variable.woff2` — **Inter**, licensed under the SIL Open Font License (OFL).
- `JetBrainsMono-Variable.woff2` — **JetBrains Mono**, licensed under Apache License 2.0.

## Why not a CDN? (D16)

- Popush is an offline-capable desktop app; fonts must work with no network.
- Loading fonts from a CDN would leak usage to a third party, contradicting the
  privacy stance (no telemetry, no external calls).

The font binaries are intentionally not committed as part of the frontend
scaffolding task; drop the licensed `.woff2` files here before shipping.
