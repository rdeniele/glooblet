# Glooblet

A lightweight Windows desktop app for mouse optimization, per-game sensitivity profiles, and
desktop click-sound personalization — built with Tauri 2, Rust, React, and SQLite.

Glooblet configures settings; it does not automate gameplay. See [docs/SECURITY.md](docs/SECURITY.md)
for the full anti-cheat / game-safety policy and [docs/STATUS.md](docs/STATUS.md) for exactly what's
implemented versus deferred in this build.

**[Download the Windows installer](releases/Glooblet-Setup.exe)** (Windows 10/11, 64-bit, installs
for your user account, no admin rights needed), or see the [landing page](landing/index.html)
(`landing/`) for a shareable download page — see [landing/README.md](landing/README.md) for how to
preview or deploy it. Licensed under [MIT](LICENSE).

## Requirements

- Windows 10/11
- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/tools/install) (stable, MSVC toolchain)
- Microsoft Visual Studio Build Tools (C++ build tools workload) — required by `rusqlite`'s
  bundled SQLite and by Tauri itself
- WebView2 Runtime (preinstalled on current Windows 10/11)

## Development

```bash
npm install
npm run tauri dev
```

This starts the Vite dev server and launches the native window with hot reload on the frontend.
Rust changes trigger a recompile + relaunch.

## Building

```bash
npm run tauri build
```

Produces an NSIS installer at `src-tauri/target/release/bundle/nsis/Glooblet_0.1.0_x64-setup.exe`.
Requires NSIS to be installed on the build machine (`winget install NSIS.NSIS`); without it, the
bundling step fails but `src-tauri/target/release/glooblet.exe` (the raw, portable binary) still
gets built.

## Tests

Backend logic (sensitivity math, DB persistence/migrations, profile validation, audio file
validation) is covered by Rust unit tests:

```bash
cd src-tauri
cargo test
```

Frontend type-checking:

```bash
npm run build   # runs `tsc` then `vite build`
```

There is no separate frontend unit-test suite — the frontend is a thin view over the Rust
commands, and the logic worth testing (math, validation, persistence) lives in Rust. See
[docs/STATUS.md](docs/STATUS.md) for what manual verification this build has and hasn't had.

## Project layout

```
src/                      React + TypeScript frontend
  components/              Shared UI (Sidebar, dialogs, form controls) and page-specific pieces
  pages/                   The six app sections (Overview, Mouse Settings, Game Profiles,
                            Click Studio, Devices, Settings)
  lib/                     Typed API wrapper over Tauri's invoke(), shared types, toast system

src-tauri/src/
  commands/                #[tauri::command] handlers — the only code the frontend can call
  windows_integration/      Pointer settings, startup registry, foreground-window lookup,
                            device enumeration — all via documented Win32 APIs
  audio/                    Non-blocking sound playback (rodio) + the optional global click hook
  db/                       SQLite access and schema migrations
  calc.rs                   Sensitivity / cm-per-360° math, with per-game confidence levels
  registry.rs               Static game compatibility registry
  models.rs                 Shared data types (mirrored by hand in src/lib/types.ts)

src-tauri/resources/sounds/ Procedurally generated, original click-sound WAVs (no licensed audio)
```

## Data storage

Everything is local: a SQLite database and custom sound files live under the OS app-data
directory for Glooblet (Windows: `%APPDATA%\com.glooblet.app\`). No account, no cloud sync, no
network access required for any core feature.
