# Glooblet landing page

A static marketing page: plain HTML/CSS, no build step, no dependencies, no JavaScript. Its one
job is to get someone from "what's this" to downloading the actual Windows app. It's deliberately
kept separate from the Tauri app's own frontend (`src/`).

## Sections

Hero (what it is + download CTA) → Features (what each of the six app screens actually does) →
Safety (the anti-cheat policy, in full) → Download (the real download link) → Footer.

## Preview locally

Open `index.html` directly in a browser, or serve it with any static file server:

```bash
cd landing
python3 -m http.server 4173
```

## Deploying

This is a plain static site, so any static host works (GitHub Pages, Netlify, Vercel, Cloudflare
Pages, S3). For GitHub Pages from this repo: Settings → Pages → Deploy from a branch → `main` →
`/landing`.

## Assets

- `assets/mascot/`: brand images copied from `src/assets/mascot/` (except `wordmark-full.webp`,
  a larger hero crop of the original brand sheet, converted to WebP to keep it small).
- `assets/icon.png`: same app icon as `public/icon.png`.

These are copies, not symlinks or build artifacts. If the source assets change, re-copy the ones
that matter here.
