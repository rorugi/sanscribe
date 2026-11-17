# Sanscribe

Select Sanskrit text in any Windows application, press a hotkey, and it is
transliterated in place. Runs fully offline, lives in the system tray.

| Hotkey (default)     | Action |
| -------------------- | ------ |
| `Ctrl+Shift+Alt+T`   | **Toggle**: संस्कृतम् → saṃskṛtam → संस्कृतम् (the result stays selected, so you can press again) |
| `Ctrl+Shift+Alt+A`   | **Insert alongside**: संस्कृतम् → संस्कृतम् (saṃskṛtam). Works from Latin too. Pressing again on the result does nothing. |

Both hotkeys and the alongside format can be changed in the settings window
(left-click the tray icon).

## Settings

- **Standard**: IAST (ṃ, ṛ, e, o) or ISO 15919 (ṁ, r̥, ē, ō)
- **Nukta letters** (for Hindi): क़ ख़ ग़ ज़ ड़ ढ़ फ़ य़ → q k͟h ġ z ṛ ṛh f ẏ. When off, the nukta is ignored (क़ → k).
  In IAST mode, ड़ also becomes ṛ (same as ऋ), so on the way back ṛ always becomes ऋ. ISO 15919 has no such clash.
- **Convert digits**: १२३ ⇄ 123
- **Read selection via UI Automation**: reads the selection without the clipboard in apps that support it
  (Word, Outlook, Notepad, most browsers); falls back to Ctrl+C otherwise.
- **Insert by pasting or typing**: pasting is fast and the clipboard is restored afterwards; typing never touches
  the clipboard, but Word's AutoCorrect may change the text.
- **Start at sign-in**, **check for updates at startup**

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
`src-tauri\target\release\bundle\nsis\Sanscribe_0.1.0_x64-setup.exe`; it installs per user
and needs no admin rights.

Engine tests only: `cargo test --manifest-path core/Cargo.toml`

## Releases and updates

Installed copies check
`https://github.com/rorugi/sanscribe/releases/latest/download/latest.json`
at startup (and via the tray menu). If a newer version is published, they offer to install it.

To publish a new version:

```powershell
.\release.ps1 0.2.0
```

This sets the version, commits, tags `v0.2.0` and pushes. GitHub Actions then builds and
signs the installer and creates a **draft** release containing the installer, its signature
and `latest.json`. Download and test it, then press **Publish release** on GitHub.
Installed copies only see an update once the release is published.

One-time setup: the release workflow needs two repository secrets
(GitHub → Settings → Secrets and variables → Actions):

| Secret | Value |
| --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY` | contents of `.secrets\sanscribe-updater.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | contents of `.secrets\sanscribe-updater.key.password` |

**Back up the `.secrets` folder.** It is not in git. Without the key and its password you
can't publish updates that existing installations will accept.

Every push to `main` is also built and tested on Windows by the CI workflow.

## Project layout

```
core/            transliteration engine (pure Rust, unit-tested)
src-tauri/       the app: hotkeys, selection/clipboard, tray, settings, updater
  src/win/       Windows integration (UI Automation, clipboard, SendInput)
ui/              settings window (HTML/CSS/JS)
```

## Known limitations

- Apps running **as administrator** can't receive input from a normal app (Windows security).
- If an app doesn't support UI Automation, Sanscribe uses Ctrl+C. Some editors (e.g. VS Code)
  copy the whole line when nothing is selected, so select text before pressing the hotkey.
- After a toggle, the result is re-selected via UI Automation. Where that isn't available, it
  falls back to Shift+← for Latin results only, so in such apps re-select Devanagari results by hand.
- Vedic accent marks are passed through unchanged.
- macOS support comes later (the engine and settings are already cross-platform).
