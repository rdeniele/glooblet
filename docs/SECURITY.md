# Security and anti-cheat policy

## What Glooblet does not do

Glooblet is a mouse-configuration and personalization utility. It does **not**, and will not,
implement:

- Aimbots, automatic crosshair movement, or any target-detection that controls aim
- Recoil compensation or triggerbots
- Automated shooting or gameplay-action macros
- Reading or writing game process memory
- DLL or code injection into any process
- Process manipulation or attaching to protected/anti-cheat processes
- Anti-cheat bypasses or detection evasion of any kind
- Kernel-level input interception or custom drivers
- Game executable patching
- Undocumented manipulation of protected game settings
- Anything designed to conceal its own behavior from anti-cheat software

Everything Glooblet does is visible, user-mode, and built from documented Windows APIs. There is
nothing in this codebase that reads from or writes to another process's memory, and nothing that
installs a kernel driver.

## What Glooblet does

| Feature | Mechanism | Scope |
|---|---|---|
| Pointer speed / "Enhance pointer precision" | `SystemParametersInfoW` (`SPI_GETMOUSESPEED`/`SPI_SETMOUSESPEED`/`SPI_GETMOUSE`/`SPI_SETMOUSE`) | Global Windows setting, confirmed via read-back after every write |
| "Launch at startup" | Per-user Run registry key (`HKCU\...\CurrentVersion\Run`) | No admin rights, no scheduled task, no service |
| Mouse device identification | SetupAPI (`SetupDiGetClassDevsW` + `SetupDiGetDeviceRegistryPropertyW`) scoped to the Mouse device class | Read-only; no raw HID handle is opened, no vendor protocol is spoken |
| Foreground-app lookup | `GetForegroundWindow` + `QueryFullProcessImageNameW` | Read-only process **name** lookup, used only to mute desktop click sounds over a recognized game; never reads that process's memory |
| Desktop click sounds | A `WH_MOUSE_LL` low-level hook that *observes* button-down events | `CallNextHookEx` is always called, unmodified, by every code path — Glooblet never consumes, delays, or alters a click. Off by default; a master switch stops it immediately. |

The click-sound hook is the one piece of this app that listens system-wide, so it gets extra
scrutiny: see the safety comment block at the top of
`src-tauri/src/audio/hook.rs` for the specific invariants the code is held to (always forward the
event, never block, never panic across the FFI boundary).

## Game compatibility

Glooblet never applies a setting *inside* a game process. Every game profile is "manual
configuration only": Glooblet stores the DPI/sensitivity/notes you give it, and you apply them in
that game's own settings menu. `src-tauri/src/registry.rs` is the versioned compatibility
registry backing the in-app "Compatibility" panel — it exists so the app can say exactly that,
per game, instead of leaving the question implicit. See [docs/STATUS.md](STATUS.md) for the full
table and review dates.

**No guarantee of acceptance.** No third-party tool can guarantee that a given anti-cheat system
accepts it running alongside a game. Glooblet minimizes the risk by staying entirely out of game
processes, but that is a risk-reduction design choice, not a publisher-issued guarantee. The
in-app first-run notice and Settings page both state this plainly.

## Tauri / application security

- The frontend's only native capabilities come from `src-tauri/capabilities/default.json`:
  `core:default` plus `dialog:allow-open` / `dialog:allow-save` (used for profile import/export
  and picking a sound file to import). There is no filesystem, shell, or HTTP plugin wired in —
  file reads/writes happen only inside Rust commands that validate and scope every path
  themselves (custom sounds are copied into an app-controlled directory under a generated UUID
  filename; exports write to exactly the path the OS save dialog returned).
- Every `#[tauri::command]` validates its own input (see `db/profiles.rs::validate`,
  `audio::validate_audio_file`) rather than trusting the frontend. Imported profile JSON is
  re-validated through the same path as a manually created profile.
- The window's CSP (`src-tauri/tauri.conf.json`) restricts script/style/font/image/media sources;
  there's no remote script loading.
- No credentials, tokens, or payment data are ever handled — Glooblet has no accounts and no
  network calls for its core features.
- Errors cross the IPC boundary as a typed `{ code, message }` payload (`src-tauri/src/error.rs`)
  rather than raw Rust error strings/panics, so the frontend can react to failure categories
  (not found, invalid input, unsupported, permission denied, ...) without parsing text.

## Reporting a concern

This is a sample/demo build produced in a single engineering pass, not an audited release. Before
distributing it, have someone re-review `audio/hook.rs` and the Windows-integration modules
specifically — anything that calls into `unsafe` Win32 APIs deserves a second set of eyes beyond
what shipped here.
