# popush.dev

The Popush landing page. A **Vite + React + TypeScript** app that reuses the
desktop app's own design system: the pink-on-near-black tokens, hard offset
shadows, sharp corners, an 8-bit Silkscreen display face, Inter for body, and
JetBrains Mono for code.

```
website/
  index.html              Vite entry (also carries the Content-Security-Policy)
  vite.config.ts          builds to ONE self-contained index.html
  src/
    main.tsx              mounts <App/>, sets the favicon from the logo
    App.tsx               page composition + scroll-reveal
    index.css             the Popush tokens + component styles
    hooks/useTheme.ts     dark/light toggle, mirrors the app
    hooks/useReveal.ts    IntersectionObserver scroll-in animation
    lib/site.ts           links + the install command in one place
    components/           Header, Hero, Terminal, Features, HowItWorks, Download, Footer, Icons, CopyButton
    assets/               logo.svg + Inter/JetBrains Mono/Silkscreen woff2 (self-hosted)
```

## Develop

```bash
cd website
npm install
npm run dev      # http://localhost:5173
```

## Build

```bash
npm run build    # typechecks, then emits dist/index.html
```

`vite-plugin-singlefile` plus an unlimited asset-inline limit build the whole
page into a **single self-contained `dist/index.html`**: JS, CSS, fonts, and the
logo are all inlined, so there are zero external requests and nothing to
allow-list.

## Deploy to popush.dev

Serve `dist/index.html`. Any static host works.

- **Vercel / Netlify / Cloudflare Pages:** project root `website/`, build
  `npm run build`, output `dist`.
- **Your own VPS (Caddy):** the same Caddy already fronting your sites can serve
  it. Drop `dist/index.html` somewhere like `/srv/popush.dev`:

  ```caddy
  popush.dev, www.popush.dev {
      root * /srv/popush.dev
      file_server
      encode zstd gzip

      # Security headers. The page itself makes no network requests, so this
      # policy is strict on purpose.
      header {
          Content-Security-Policy "default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; img-src 'self' data:; font-src data:; connect-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'; upgrade-insecure-requests"
          Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
          X-Content-Type-Options "nosniff"
          X-Frame-Options "DENY"
          Referrer-Policy "strict-origin-when-cross-origin"
          Permissions-Policy "camera=(), microphone=(), geolocation=(), payment=(), usb=(), interest-cohort=()"
          Cross-Origin-Opener-Policy "same-origin"
          Cross-Origin-Resource-Policy "same-origin"
          -Server
      }

      # Make `curl -fsSL popush.dev/install | bash` work by redirecting to the
      # installer in the repo (or copy get-popush.sh in and serve it directly).
      redir /install https://raw.githubusercontent.com/twosteppy/Popush/main/get-popush.sh
  }
  ```

`frame-ancestors 'none'` only takes effect from the response header, not the
in-page meta tag, so keep it in the Caddy block above to block clickjacking.
