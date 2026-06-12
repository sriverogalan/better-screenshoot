# Better Screenshoot

Screenshots with a built-in editor, global shortcuts, and history. **macOS only** for now.

## Download

**[Latest release](https://github.com/sriverogalan/better-screenshoot/releases/latest)**

| Platform | File | Requirements |
|---|---|---|
| macOS (Apple Silicon) | `Better Screenshoot_*_aarch64.dmg` | macOS 12 or later |
| macOS (Intel) | `Better Screenshoot_*_x64.dmg` | macOS 12 or later |

## Installation

1. Download the `.dmg` that matches your Mac (Apple Silicon or Intel).
2. Open the `.dmg` and drag **Better Screenshoot** to **Applications**.
3. Open the app. If macOS shows a security warning, see [Troubleshooting](#troubleshooting).
4. Grant **Screen Recording** permission when prompted (System Settings → Privacy & Security → Screen Recording).
5. For reliable global shortcuts, also enable **Accessibility** if the app requests it.

## Getting started

Better Screenshoot lives in the **system tray**. From there you can capture, open history, or go to settings.

After capturing, the **editor** opens so you can annotate the image: arrows, rectangles, text, highlight, freehand stroke, and blur. When you're done, the capture is copied to the clipboard and you can save it to disk.

### Default shortcuts

| Action | Shortcut |
|---|---|
| Capture area | ⌘⇧X |
| Capture screen | ⌘⇧⌥S |
| Capture window | ⌘⇧⌥W |
| Open history | ⌘⇧H |

You can change shortcuts in **Settings** inside the app.

## Troubleshooting

### macOS won't open the app

Current builds are not signed with an Apple certificate. It's normal to see *"can't be opened because it is from an unidentified developer"*.

**Option A:** right-click the app → **Open** → confirm.

**Option B:** System Settings → **Privacy & Security** → **Open Anyway**.

### Capture doesn't work on macOS

Go to **System Settings → Privacy & Security → Screen Recording** and enable Better Screenshoot. Restart the app if it was already open.

### Global shortcuts don't respond

On macOS, add Better Screenshoot under **Settings → Privacy & Security → Accessibility**.

## Publishing a new release

Releases are built **locally on macOS** (faster than CI). GitHub Actions only runs tests and typechecks on PRs.

1. Bump the version in the root `package.json` (CI syncs the rest on your PR).
2. Merge to `main`, then tag from `main`:

```bash
git tag v0.2.1
git push origin v0.2.1   # tag only — does not trigger a remote build
```

3. From **`main`**, build, sign, and upload the release from your Mac:

```bash
git checkout main
git pull origin main
pnpm release:mac v0.2.1 --all-arch
```

The release script refuses to run on any other branch.

This creates a **draft** GitHub release with `.dmg` files, signed updater bundles (`.tar.gz`), and `latest.json`. Review it on GitHub and publish when ready — installed apps only pick up updates from **published** releases.

Options:

- `--all-arch` — Apple Silicon + Intel (omit to build only for your current Mac)
- `--publish` — publish immediately instead of a draft

Signing key: `.tauri/better-screenshoot.key` (gitignored). A backup copy is stored in the repo secret `TAURI_SIGNING_PRIVATE_KEY`. Generate a new pair only if you lose the private key.

See [docs/branching.md](docs/branching.md) for branch flow and `main` protection.

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

Build the macOS installer:

```bash
pnpm build:mac      # .dmg (macOS only)
```

The `.dmg` is generated in `apps/desktop/src-tauri/target/release/bundle/dmg/`.

### Structure

```
apps/desktop/          # Tauri + Vue app
packages/capture-core/ # Rust capture engine
packages/shared-types/ # Shared IPC types
packages/licensing/    # Open core tiers + license validation
packages/raycast-extension/ # Raycast extension (macOS)
cli/                   # better-screenshoot CLI
docs/api.md            # URL scheme and CLI
```

### Integrations

```bash
open "betterscreenshoot://capture-area"
better-screenshoot-cli open capture-area
```

See [docs/api.md](docs/api.md).

### Branches and releases

Workflow, `main` protection, and versioning: [docs/branching.md](docs/branching.md).

## License

Open core — Community (OSS) + paid Pro/Cloud/Team tiers.
