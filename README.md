# Sanscribe

[![Latest release](https://img.shields.io/github/v/release/rorugi/sanscribe?label=latest%20version)](https://github.com/rorugi/sanscribe/releases/latest)

**Transliterate Sanskrit in any Windows application.** Select Devanagari text, press a
hotkey, and it becomes IAST. Press it again and it's Devanagari again. Works in Word, Outlook,
your browser, Notepad and most other programs. It runs completely offline.

संस्कृतम् ⇄ saṃskṛtam

## Download

### [⬇ Download Sanscribe 0.2.1 for Windows](https://github.com/rorugi/sanscribe/releases/download/v0.2.1/Sanscribe_0.2.1_x64-setup.exe)

Windows 10 or 11 (64-bit) · free · [what's new and older versions](https://github.com/rorugi/sanscribe/releases)

Already installed? Sanscribe updates itself, see [Updates](#updates).

## Installation

1. Run the downloaded setup file. You don't need administrator rights.
2. If Windows shows **"Windows protected your PC"**, click **More info → Run anyway**.
   This appears because the installer isn't signed with a paid Microsoft certificate.
3. Sanscribe starts and its icon <img src="src-tauri/icons/32x32.png" width="16" alt=""> appears in
   the system tray, at the bottom right of the taskbar (possibly hidden under the **^** arrow).
   The first time, the settings window opens so you can check the hotkeys.

Sanscribe starts automatically when you sign in to Windows. You can turn that off in the settings.

## How to use

1. **Select** the text you want to convert.
2. Press a **hotkey**:

| Hotkey | What it does | Example |
| --- | --- | --- |
| **Ctrl + Shift + Alt + T** | **Toggle**: replaces the selection with the other script. The result stays selected, so pressing again converts it back. | संस्कृतम् → saṃskṛtam → संस्कृतम् |
| **Ctrl + Shift + Alt + A** | **Insert alongside**: keeps the original and adds the transliteration. | संस्कृतम् → संस्कृतम् (saṃskṛtam) |

Both work in either direction (Devanagari → Latin and Latin → Devanagari). If the selection
contains any Devanagari, the Devanagari is converted and other text is left as it is.

You can change both hotkeys in the settings.

## Typing diacritics

Turn on **Diacritic keys** in the settings to type IAST letters directly in any application.
Press the key combination first, then the letter:

| Keys | Mark | Letters |
| --- | --- | --- |
| **AltGr + .** | dot below | ṛ ṭ ḍ ṇ ṣ ṃ ḥ ḷ |
| **AltGr + Shift + .** | dot above | ṅ ṁ |
| **AltGr + -** | line above | ā ī ū |
| **AltGr + Shift + -** | line below | ḻ ṉ ṟ |
| **AltGr + ,** | acute | ś |
| **AltGr + Shift + ,** | tilde | ñ |

Example: **AltGr + -**, then **a** → ā. For **ṝ** and **ḹ** press both marks before the letter:
**AltGr + .**, **AltGr + -**, then **r**. Shift + letter gives capitals (Ā). **Esc** cancels.
Keyboards without AltGr can use **Ctrl + Alt** instead.

## Settings

Left-click the tray icon (or right-click → **Settings…**).

- **Standard**: IAST or ISO 15919

  | Devanagari | IAST | ISO 15919 |
  | --- | --- | --- |
  | संस्कृतम् | saṃskṛtam | saṁskr̥tam |
  | कृष्ण | kṛṣṇa | kr̥ṣṇa |
  | देवो | devo | dēvō |
  | ॐ नमः शिवाय | oṃ namaḥ śivāya | ōṁ namaḥ śivāya |

- **Nukta letters** (for Hindi): क़ ख़ ग़ ज़ ड़ ढ़ फ़ → q k͟h ġ z ṛ ṛh f. When off, the dot is ignored, as usual in Sanskrit (क़ → k).
- **Convert digits**: १२३ ⇄ 123
- **Diacritic keys**: see [Typing diacritics](#typing-diacritics) (off by default)
- **Alongside format**: संस्कृतम् (saṃskṛtam), संस्कृतम् [saṃskṛtam], संस्कृतम् / saṃskṛtam, on a new line, or your own.
- **Try it**: type or paste text to see the result before using it.

Transliteration follows Sanskrit conventions: the inherent *a* is always written (राम → rāma),
and daṇḍas become | and ||.

## Updates

Sanscribe checks for a new version when it starts and asks before installing it.
You can also check any time: right-click the tray icon → **Check for updates**.

## Good to know

- **Privacy:** everything happens on your computer. The only internet access is the update check.
- **Clipboard:** where an application doesn't let Sanscribe read the selection directly, it briefly
  uses the clipboard. Your clipboard is restored afterwards, and Sanscribe's own entries don't
  appear in the clipboard history (Win + V).
- **Select first:** some editors (e.g. VS Code) copy the whole line when nothing is selected.
- **Programs running as administrator** can't be controlled by normal programs, so Sanscribe
  doesn't work in them.
- In a few applications the result can't be re-selected automatically after a Devanagari
  conversion. Select it again by hand to convert it back.
- Nothing happens / a short sound? Nothing was selected, or the selection had nothing to convert.

## Uninstall

Windows **Settings → Apps → Installed apps → Sanscribe → Uninstall**.

## Feedback

Found a wrong transliteration or a program where it doesn't work?
[Open an issue](https://github.com/rorugi/sanscribe/issues) with the text and the program you used.

## Acknowledgements

Inspired by [SansTrans](https://t-kintaert.github.io/sanstrans/) by Thomas Kintaert
(Institut für Südasien-, Tibet- und Buddhismuskunde, Universität Wien), a free Windows tool
for typing and converting Sanskrit transliteration. Thank you!

## License

Sanscribe is free software under the [MIT License](LICENSE).

---

Developers: building, testing and releasing are described in [DEVELOPMENT.md](DEVELOPMENT.md).
