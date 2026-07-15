# popush.dev

The Popush landing page — a **Vite + React + TypeScript** app that reuses the
desktop app's own design system (the pink-on-near-black tokens, hard offset
shadows, sharp corners, Inter + JetBrains Mono).

```
website/
  index.html              Vite entry
  vite.config.ts          builds to ONE self-contained index.html
  src/
    main.tsx              mounts <App/>, sets the favicon from the logo
    App.tsx               page composition
    index.css             the Popush tokens + component styles
    hooks/useTheme.ts     dark/light toggle, mirrors the app
    lib/site.ts           links + the install command in one place
    components/           Header, Hero, PipelineDemo, Features, HowItWorks, Download, Footer, Icons, CopyButton
    assets/               logo.svg + Inter/JetBrains Mono woff2 (self-hosted)
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

`vite-plugin-singlefile` plus an unlimited asset-inline limit means the build is
a **single self-contained `dist/index.html`** — JS, CSS, fonts, and the logo are
all inlined, so there are zero external requests and nothing to allow-list.

## Deploy to popush.dev

Serve `dist/index.html`. Any static host works:

- **Vercel / Netlify / Cloudflare Pages:** project root `website/`, build
  `npm run build`, output `dist`.
- **Your own VPS (Caddy):** the same Caddy already fronting your sites can serve
  it — drop `dist/index.html` somewhere like `/srv/popush.dev`:

  ```caddy
  popush.dev, www.popush.dev {
      root * /srv/popush.dev
      file_server
      encode zstd gzip
      # make `curl -fsSL popush.dev/install | bash` work
      redir /install https://raw.githubusercontent.com/twosteppy/Popush/main/get-popush.sh
  }
  ```
