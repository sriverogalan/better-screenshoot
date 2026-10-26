<p align="center">
  <img src="docs/assets/icon.png" width="128" height="128" alt="Better Screenshoot icon">
</p>

<h1 align="center">Better Screenshoot</h1>

<p align="center">
  <strong>Screenshots with a built-in editor, global shortcuts, and history.</strong><br>
  macOS only · lives in the menu bar · annotate and share in seconds.
</p>

<p align="center">
  <a href="#install"><img src="https://img.shields.io/badge/macOS-12%2B-black?style=flat-square" alt="macOS 12+"></a>
  <a href="#install"><img src="https://img.shields.io/badge/Homebrew-cask-0A84FF?style=flat-square" alt="Homebrew"></a>
  <a href="https://github.com/sriverogalan/better-screenshoot/issues/new/choose"><img src="https://img.shields.io/badge/Feedback-open%20an%20issue-success?style=flat-square" alt="Feedback"></a>
</p>

<p align="center">
  <img src="docs/assets/hero.png" alt="Better Screenshoot product hero" width="920">
</p>

---

## Demo

<p align="center">
  <img src="docs/assets/demo.gif" alt="Better Screenshoot demo: onboarding, menu bar, history, settings, and editor" width="920">
</p>

<details>
<summary>MP4 version (smaller download)</summary>

[Download demo.mp4](docs/assets/demo.mp4)

</details>

## Install

```sh
brew install --cask sriverogalan/tap/better-screenshoot
```

Uninstall:

```sh
brew uninstall --cask better-screenshoot
```

> **Requirements:** macOS 12.0 (Monterey) or later · **Screen Recording** permission (the app walks you through it on first launch).

## Features

| | |
|---|---|
| **Fast capture** | Region or full screen from the menu bar or global shortcuts. |
| **Instant editor** | Arrows, rectangles, text, highlight, freehand, and blur. |
| **History** | Keep what matters and reopen it later. |
| **Your shortcuts** | Remap hotkeys in Settings — optionally replace macOS system captures. |

## Screenshots

<p align="center">
  <img src="docs/assets/screenshot-menubar.png" alt="Menu bar panel" width="280">
  &nbsp;
  <img src="docs/assets/screenshot-history.png" alt="History view" width="420">
</p>

<p align="center">
  <img src="docs/assets/screenshot-settings.png" alt="macOS-style settings" width="720">
</p>

## Default shortcuts

| Action | Shortcut |
|---|---|
| Capture region | ⌘⇧X |
| Capture screen | ⌘⇧⌥S |
| Open history | ⌘⇧H |

Change them anytime in **Settings → Global shortcuts**.

## Found a bug? Open an issue

We care about real multi-monitor setups and day-to-day capture flows. If something breaks:

1. Install or update via Homebrew.
2. Reproduce once.
3. File a report → **[New issue](https://github.com/sriverogalan/better-screenshoot/issues/new/choose)**

Please include when you can: macOS version, number of displays, and whether you use app shortcuts or system capture mode.

### Troubleshooting

**Capture does nothing**  
System Settings → Privacy & Security → Screen Recording → enable Better Screenshoot. Restart the app if it was already open.

**Global shortcuts do not respond**  
System Settings → Privacy & Security → Accessibility → add Better Screenshoot.

---

## For developers

**Stack:** Tauri 2 · Vue 3 · TypeScript · Tailwind · Rust (`capture-core`) · Pinia · SQLite

```bash
pnpm install
pnpm dev
```

```
apps/desktop/          # Tauri + Vue app
packages/capture-core/ # Native capture engine
packages/shared-types/ # Shared IPC types
packages/licensing/    # Open core + license validation
packages/better-screenshoot/ # Raycast extension
cli/                   # better-screenshoot CLI
docs/api.md            # URL scheme and CLI
```

Integrations:

```bash
open "betterscreenshoot://capture-area"
better-screenshoot-cli open capture-area
```

Raycast extension: `packages/better-screenshoot/` — `pnpm raycast:dev`.  
API: [docs/api.md](docs/api.md) · branching: [docs/branching.md](docs/branching.md)

### Publishing a release

1. Bump the root `package.json` version and open a PR to `develop`.
2. Merge `develop` → `main` (CI must pass).
3. On `main`, build the DMG: `cd apps/desktop && pnpm tauri build`
4. Upload to `sriverogalan/better-screenshoot-releases` and update the cask in `sriverogalan/homebrew-tap`.

## License

Open core — Community (OSS) + paid Pro / Cloud / Team tiers.
