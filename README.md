# Better Screenshoot

Screenshots with a built-in editor, global shortcuts, and history. **macOS only** for now.

## Install

```sh
brew install --cask sriverogalan/tap/better-screenshoot
```

To uninstall:

```sh
brew uninstall --cask better-screenshoot
```

> **Requirements:** macOS 12.0 (Monterey) or later · Screen Recording permission (the app guides you through it on first launch)

## Use the app

Better Screenshoot opens on launch and lives in the **menu bar**. From there you can capture, open history, or go to settings.

After capturing, the **editor** opens so you can annotate: arrows, rectangles, text, highlight, freehand stroke, and blur. The capture is copied to the clipboard and optionally saved to disk.

### Default shortcuts

| Action | Shortcut |
|---|---|
| Capture area | ⌘⇧X |
| Capture screen | ⌘⇧⌥S |
| Capture window | ⌘⇧⌥W |
| Open history | ⌘⇧H |

Shortcuts can be changed in **Settings** inside the app.

## Troubleshooting

### Capture doesn't work

Go to **System Settings → Privacy & Security → Screen Recording** and enable Better Screenshoot. Restart the app if it was already open.

### Global shortcuts don't respond

Add Better Screenshoot under **System Settings → Privacy & Security → Accessibility**.

---

## For developers

### Stack

- **Tauri 2** + **Vue 3** + **TypeScript** + **Tailwind CSS**
- **Rust** (`capture-core`) for native capture
- **Pinia** for state, **SQLite** for history

### Local development

```bash
pnpm install
pnpm dev
```

### Structure

```
apps/desktop/          # Tauri + Vue app
packages/capture-core/ # Rust capture engine
packages/shared-types/ # Shared IPC types
packages/licensing/    # Open core tiers + license validation
packages/better-screenshoot/ # Raycast extension
cli/                   # better-screenshoot CLI
docs/api.md            # URL scheme and CLI
```

### Integrations

```bash
open "betterscreenshoot://capture-area"
better-screenshoot-cli open capture-area
```

Raycast extension: `packages/better-screenshoot/` — local dev: `pnpm raycast:dev`.

See [docs/api.md](docs/api.md).

### Publishing a new release

1. Bump version in root `package.json` and open a PR to `develop`.
2. Merge `develop` → `main` via PR (CI must pass).
3. On `main`, build the DMG locally:

```bash
cd apps/desktop && pnpm tauri build
```

4. Compute SHA256 and upload to `sriverogalan/better-screenshoot-releases`:

```bash
shasum -a 256 target/release/bundle/dmg/*.dmg
gh release create vX.Y.Z target/release/bundle/dmg/*.dmg --repo sriverogalan/better-screenshoot-releases
```

5. Update `homebrew-tap/Casks/better-screenshoot.rb` with new version + SHA256 and push to `sriverogalan/homebrew-tap`.

## License

Open core — Community (OSS) + paid Pro/Cloud/Team tiers.
