# Implementation status

Honest accounting of what this build actually does versus what the original spec asked for and
what's deferred. Written at the end of the build pass that produced this codebase; update it when
behavior changes.

## Implemented and real (not mocked)

- **Windows pointer settings** — reads and writes actual `SPI_GETMOUSESPEED`/`SPI_SETMOUSESPEED`/
  `SPI_GETMOUSE`/`SPI_SETMOUSE` values, verifies every write by reading it back, and
  auto-rolls-back if Windows didn't accept the change. Confirmation dialog before every apply.
- **Game profiles** — full CRUD, duplicate, JSON import/export (via native file dialogs),
  validated both on manual entry and on import, persisted in SQLite with real migrations.
- **cm/360° calculator** — real math (`src-tauri/src/calc.rs`), unit tested against hand
  calculations, with an explicit `confidence` rating (`high` / `community` / `not_applicable`)
  per game and a custom-yaw-constant override for anything not in the list.
- **Click Studio sound library** — 12 original, procedurally synthesized WAV files (no licensed
  audio), real non-blocking playback via `rodio`, real upload validation (extension allow-list,
  5 MB size cap, 8 s duration cap, actual decode attempt), real deletion.
- **Click-sound hook** — a genuine `WH_MOUSE_LL` low-level hook that plays a sound on left/right/
  middle click-down, off by default, with a master switch and a "mute over games" default driven
  by a real foreground-process-name check.
- **Device enumeration** — real SetupAPI enumeration of the Windows Mouse device class
  (name/manufacturer/connection type), read-only, no admin rights.
- **Local persistence** — SQLite with a versioned migrations table, not a mock store.
- **Launch at startup** — a real per-user Run registry entry, toggleable, no admin rights.
- **Settings page safety notice** — the spec's required first-run and Settings-page anti-cheat
  statement is shown verbatim (`src/lib/constants.ts`).
- **NSIS installer** — `releases/Glooblet-Setup.exe`, per-user install (no admin rights), WebView2
  silently downloaded/installed by the bootstrapper if missing. Built, silently installed, and
  launched from the real installed location in this environment: the resulting window's class is
  `Tauri Window` (not a crash, not the native error dialog described below), confirming the
  bundled-resource path resolution works in the actual installed layout, not just in dev.
- **Startup failures show a message instead of vanishing** — the release profile sets
  `panic = "abort"` and the windowed subsystem hides the console, so an unhandled panic during
  startup used to mean the app would silently disappear with zero explanation. `setup()` now
  collects its fallible steps (app-data dir, resource dir, database open, sound seeding) into a
  `Result` and shows a native `MessageBoxW` with the real reason before exiting if any of them
  fail, instead of panicking (`src-tauri/src/lib.rs::build_app_state` / `fatal_startup_error`).
- **Error toasts show the actual error** — every `toast.error(...)` call in the frontend used to
  pass `String(e)` on a rejected `invoke()` call. Tauri rejects with the serialized
  `{ code, message }` object our `AppError` produces, not a string, so every single error toast in
  the app showed the literal text "[object Object]" instead of anything useful. Fixed once via
  `src/lib/errors.ts::getErrorMessage`, used everywhere a command's rejection reaches the user.

Backend coverage: 20 Rust unit tests across `calc`, `db::profiles`, `db::sounds`, `db` (migrations
+ settings), and `audio` (file validation), all passing (`cargo test`). Frontend type-checks clean
(`tsc --noEmit`) and the production bundle builds (`vite build`). The release binary and the NSIS
installer were both built, and the installer was silently installed and launched from its real
install location to confirm a clean startup — see the caveat below on what that still didn't
verify (no click-by-click GUI testing was done from this environment).

## Deferred / explicitly not implemented

These are left out on purpose, consistent with the spec's own "don't fake it, say so" instruction
— not implemented silently, not stubbed to look functional:

- **Per-vendor DPI / polling-rate / onboard-profile read-write** — there is no public,
  documented, cross-vendor API for this (Logitech/Razer/SteelSeries/etc. protocols are private).
  The Devices page reports every mouse with all `set_*` capabilities `false` rather than faking a
  slider that does nothing. DPI is stored as a **reference value** on a profile instead.
- **Automatic game-profile switching** — the spec explicitly asks for this to stay disabled
  unless a specific detection method has been reviewed and approved; this build ships manual
  profile selection only. The foreground-process lookup that exists (for Click Studio's
  "mute over games") is *not* wired to profile switching.
- **Code signing / auto-update** — the NSIS installer is real and installs/launches cleanly, but
  it isn't code-signed (Windows SmartScreen will warn on first run for an unsigned, low-reputation
  binary — that's expected, not a bug) and there's no auto-update mechanism or release pipeline.
- **Frontend unit tests** — the frontend is a thin view layer over the Rust commands; the logic
  worth unit-testing (math, validation, persistence) is tested on the Rust side. No Vitest/RTL
  suite was written for the React components themselves.
- **Light theme** — the design system ships one dark theme, matching the product's gaming-utility
  identity. No light/dark toggle.
- **Full brand asset set** — only the assets actually used in-app were produced (app icon, sidebar
  mascot, a handful of expression crops for empty/first-run states, the wordmark). Monochrome and
  light-background variants from the original brand sheet were not separately exported since the
  app has no light theme to use them in; source crops live in `assets-src/` if those are needed
  later.

## Verified vs. not verified — read this before treating it as finished

This build was produced and checked from an environment with terminal/compiler access but **no
interactive GUI control** (no way to click buttons, drag sliders, or watch the window render).
What was actually checked:

- `cargo check` / `cargo build` / `cargo test` — clean, backend logic covered by tests.
- `tsc --noEmit` / `vite build` — clean.
- One `tauri dev` launch, left running long enough to confirm the process started and stayed up
  without a panic in `setup()`.

What was **not** interactively verified by this process and should be click-tested before relying
on it:

- That every button, toggle, and form actually produces the expected visual state in the running
  window (layout, truncation, focus order).
- The low-level mouse-click hook's real-world behavior — that it reliably plays sounds on physical
  clicks, correctly mutes over a real running game, and unhooks cleanly on disable/exit. The code
  follows the safety rules documented in `audio/hook.rs`, but a global input hook is exactly the
  kind of thing that deserves a human clicking a real mouse before it's trusted.
- Audio actually audible through real speakers/headphones (playback code runs and the engine
  initializes, but no one listened to it).
- The native file-picker dialogs (sound import, profile import/export) in the actual window.
- Startup-registry behavior across an actual Windows sign-in cycle.

None of this is exotic risk — it's the normal gap between "compiles and unit-tests pass" and
"a person used it" for a GUI app built without GUI access. Point someone at the running app
(`npm run tauri dev`) and have them go through each of the six pages before calling this release
-ready.

## Game compatibility registry (snapshot)

Source of truth: `src-tauri/src/registry.rs`. Every entry is `manual_configuration_only` — see
[docs/SECURITY.md](SECURITY.md) for why.

| Game | Note |
|---|---|
| Counter-Strike 2 / CS:GO | Source-engine formula well documented; verify your own cm/360 before competitive use |
| VALORANT | No settings API; Vanguard means Glooblet never touches the game process |
| Apex Legends | Source-derived engine, same manual-apply model |
| Overwatch | Yaw constant is community-sourced, not Blizzard-confirmed |
| Fortnite | No stable single conversion constant (FOV/input-curve dependent) |
| Call of Duty (modern titles) | Per-weapon ADS multipliers make one constant unreliable |
| League of Legends | Fixed top-down camera — cm/360 doesn't apply |
| Minecraft | 0–200% slider, not standardized across versions/editions |
| Custom game | Manual, with an optional user-supplied yaw constant |
