# Glooblet landing page

A static marketing page: plain HTML/CSS/JS, no build step, no dependencies. It's deliberately kept
separate from the Tauri app's own frontend (`src/`). This is public-facing marketing content, not
part of the shipped application.

## What's interactive

The "Try it right here" section is a real, in-browser demo, not screenshots:

- **Mouse Settings tab**: a pointer-speed slider and acceleration toggle with a live readout. Does
  not touch your actual Windows settings (there's no native backend on a web page).
- **Game Profiles tab**: a working cm/360° calculator using the exact same formula as
  `src-tauri/src/calc.rs` (`cm = (2.54 × 360) / (dpi × sensitivity × yaw)`), computed live as you
  type.
- **Click Studio tab**: plays the actual bundled click-sound WAVs (`assets/sounds/`, copied from
  `src-tauri/resources/sounds/`) via a plain `<audio>` element.

## Preview locally

Any static file server works, e.g.:

```bash
cd landing
python3 -m http.server 4173
```

Then open `http://localhost:4173`. Opening `index.html` directly via `file://` also works in a
real browser, but some embedded/sandboxed preview tools render local files as non-interactive
snapshots. Use a server if something looks unstyled or static.

## Deploying

This is a plain static site, so any static host works (GitHub Pages, Netlify, Vercel, Cloudflare
Pages, S3). For GitHub Pages from this repo: Settings → Pages → Deploy from a branch → `main` →
`/landing`.

## Assets

- `assets/mascot/`: brand images copied from `src/assets/mascot/` (except `wordmark-full.webp`,
  a larger hero crop of the original brand sheet, converted to WebP to keep it small).
- `assets/sounds/`: a subset of the procedurally generated click sounds from
  `src-tauri/resources/sounds/`, copied here for the Click Studio demo.
- `assets/icon.png`: same app icon as `public/icon.png`.

These are copies, not symlinks or build artifacts. If the source assets change, re-copy the ones
that matter here.
