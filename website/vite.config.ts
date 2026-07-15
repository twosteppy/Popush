import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { viteSingleFile } from 'vite-plugin-singlefile';

// The site builds to a single self-contained index.html: fonts and the logo are
// inlined as data URIs (assetsInlineLimit is effectively unlimited) and the JS
// and CSS are inlined by viteSingleFile. That makes it trivial to host anywhere
// and keeps the page free of external requests.
export default defineConfig({
  plugins: [react(), viteSingleFile()],
  build: {
    assetsInlineLimit: 100_000_000,
    cssCodeSplit: false,
    target: 'es2020',
  },
});
