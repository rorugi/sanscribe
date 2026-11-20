# Developing Sanscribe

User documentation is in [README.md](README.md).

## Prerequisites (one-time, Windows 11)

1. **Rust**: <https://rustup.rs> (installs `cargo`). Check with `cargo --version`.
2. **Microsoft C++ Build Tools**: <https://visualstudio.microsoft.com/visual-cpp-build-tools/>,
   choose the workload **"Desktop development with C++"**.
3. **Node.js LTS**: <https://nodejs.org> (only used for the Tauri command-line tool).
4. WebView2 is already part of Windows 11.

## Run and build

```powershell
cd D:\git\app-sanscribe

# development (debug build, with console log)
powershell -ExecutionPolicy Bypass -File .\dev.ps1

# installer
powershell -ExecutionPolicy Bypass -File .\build.ps1
```

The first build takes several minutes. The installer ends up in
`src-tauri\target\release\bundle\nsis\Sanscribe_<version>_x64-setup.exe`; it installs per user
and needs no admin rights. A local build needs the signing key in `.secrets\` (see below).

Engine tests only: `cargo test --manifest-path core/Cargo.toml`

Logs: `%LOCALAPPDATA%\com.sanscribe.app\logs\` (also printed to the console in dev mode).

## Releases and updates

Installed copies check
`https://github.com/rorugi/sanscribe/releases/latest/download/latest.json`
at startup and via the tray menu. If a newer version is published, they offer to install it.

To publish a new version:

```powershell
.\release.ps1 0.3.0
```

This checks that everything is committed, sets the version in `tauri.conf.json`,
`src-tauri/Cargo.toml` and `package.json`, commits, tags `v0.3.0` and pushes.
GitHub Actions (`.github/workflows/release.yml`) then builds and signs the installer and
creates a **draft** release containing the installer, its signature and `latest.json`.
Download and test it, then press **Publish release** on GitHub. Installed copies only see
an update once the release is published.

When you publish a release, the workflow `readme-link.yml` points the README's download
button at the new installer and commits that to `main` (`release.ps1` pulls it before the next release).

Version numbers: `0.2.1` for fixes, `0.3.0` for new features, `1.0.0` for the first stable
version. A new version must be higher than the previous one.

One-time setup: the release workflow needs two repository secrets
(repository → Settings → Secrets and variables → Actions):

| Secret | Value |
| --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY` | contents of `.secrets\sanscribe-updater.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | contents of `.secrets\sanscribe-updater.key.password` |

**Back up the `.secrets` folder.** It is not in git. Without the key and its password you
can't publish updates that existing installations will accept.

Every push to `main` is built and tested on Windows by the CI workflow (`ci.yml`),
including `cargo clippy -D warnings`.

## Project layout

```
core/            transliteration engine (pure Rust, unit-tested)
src-tauri/       the app: hotkeys, selection/clipboard, tray, settings, updater
  src/actions.rs what happens when a hotkey is pressed
  src/win/       Windows integration (UI Automation, clipboard, SendInput)
ui/              settings window (HTML/CSS/JS, no build step)
```

## How a hotkey press works

1. Wait until the user has released the hotkey's modifier keys (otherwise Ctrl+C would arrive as Ctrl+Alt+C).
2. Read the selection via UI Automation (TextPattern). If the app doesn't support it, save the
   clipboard, send Ctrl+C and wait for the clipboard sequence number to change.
3. Transliterate. `core::Session` remembers the last conversion, so toggling back restores the
   exact original even where Latin → Devanagari is ambiguous (अइ and ऐ are both "ai").
4. Paste the result (default) or type it as Unicode key events (setting).
5. Re-select the result via UI Automation; fallback Shift+← for Latin output only.
6. Restore the saved clipboard. Clipboard writes are flagged so that Windows clipboard history
   and cloud clipboard ignore them.

## Diacritic keys

`src-tauri/src/win/deadkeys.rs` installs a low-level keyboard hook (`WH_KEYBOARD_LL`) on its
own thread while the setting is on. AltGr (or Ctrl+Alt) with `.` `-` `,` (± Shift) is
swallowed and remembered as a mark; the next letter is swallowed too, composed with
`core::compose` (NFC, so marks can be pressed in any order) and typed as a Unicode key
event. Injected events are ignored, so the hook never sees its own output. Pending marks
expire after 5 seconds; Esc cancels; any other key cancels and passes through.

## Transliteration notes

- Input is NFC-normalised; precomposed nukta letters (U+0958–U+095F) are handled.
- In IAST mode ड़ becomes ṛ like ऋ, so on the way back ṛ always becomes ऋ. ISO 15919 writes
  ऋ as r̥, so there is no clash there.
- IAST ḷ (Latin → Devanagari) becomes ळ before a vowel (agnim īḷe) and the vowel ऌ otherwise (kḷpta).
- An apostrophe after e/o/ā and before a letter becomes avagraha (so 'ham → सोऽहम्).
- Vedic accent marks are passed through unchanged.

## Known limitations

- Apps running as administrator can't receive input from a normal app (Windows UIPI).
- macOS support is not implemented yet. The engine, settings and UI are cross-platform; the
  OS integration (`src-tauri/src/win/`) needs a macOS counterpart (Accessibility API, CGEvent).
