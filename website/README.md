# popush.dev

The Popush landing page. A single self-contained static folder — no build step,
no dependencies.

```
website/
  index.html        the page
  logo.svg          the Popush mark
  fonts/            Inter + JetBrains Mono (self-hosted, no CDN)
```

## Deploy

Any static host works. Point it at this folder and serve `index.html`.

- **Vercel / Netlify / Cloudflare Pages:** set the project root to `website/`.
- **GitHub Pages:** publish the `website/` folder.
- **Your own VPS (Caddy):** the same Caddy already fronting your sites can serve
  it. Add a block, dropping the folder somewhere like `/srv/popush.dev`:

  ```caddy
  popush.dev, www.popush.dev {
      root * /srv/popush.dev
      file_server
      encode zstd gzip

      # Make `curl -fsSL popush.dev/install | bash` work by redirecting to the
      # installer in the repo (or copy get-popush.sh in and serve it directly).
      redir /install https://raw.githubusercontent.com/twosteppy/Popush/main/get-popush.sh
  }
  ```

Everything is same-origin (fonts and logo are local), so there are no external
requests and nothing to allow-list.

> `artifact.html` is a generated single-file bundle for previews only — not
> part of the deployed site.
