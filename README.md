# Better Screenshoot

Screenshots with a built-in editor, global shortcuts, and history. **macOS only** for now.

## Get started

### 1. Download

**[Latest release](https://github.com/sriverogalan/better-screenshoot/releases/latest)**

| Platform | File | Requirements |
|---|---|---|
| macOS (Apple Silicon) | `Better Screenshoot_*_aarch64.dmg` | macOS 12 or later |
| macOS (Intel) | `Better Screenshoot_*_x64.dmg` | macOS 12 or later |

### 2. Install on macOS

Better Screenshoot is **free and open source**. It is not signed with a paid Apple Developer certificate, so macOS shows a **one-time** security prompt on first launch. This is expected.

1. Open the `.dmg` and drag **Better Screenshoot** to **Applications**.
2. **First launch only:** Finder → **Applications** → **right-click** (or Control-click) **Better Screenshoot** → **Open** → confirm **Open**.  
   Do **not** double-click the first time — macOS will block it with a message like *"Apple cannot verify that Better Screenshoot.app…"* / *"Apple no ha podido verificar…"*.
3. Allow **Screen Recording** when prompted (required to capture).
4. Allow **Accessibility** if prompted (required for global shortcuts).

**Español — primera apertura:** **Aplicaciones** → **clic derecho** en Better Screenshoot → **Abrir** → **Abrir**. Solo la primera vez.

Still blocked? See [Troubleshooting](#troubleshooting).

### 3. Use the app

Better Screenshoot lives in the **menu bar** (system tray). From there you can capture, open history, or go to settings.

After capturing, the **editor** opens so you can annotate the image: arrows, rectangles, text, highlight, freehand stroke, and blur. When you're done, the capture is copied to the clipboard and you can save it to disk.

#### Default shortcuts

| Action | Shortcut |
|---|---|
| Capture area | ⌘⇧X |
| Capture screen | ⌘⇧⌥S |
| Capture window | ⌘⇧⌥W |
| Open history | ⌘⇧H |

You can change shortcuts in **Settings** inside the app.

## Troubleshooting

### macOS won't open the app

Unsigned builds always show a Gatekeeper warning on first launch. This is not malware — the app is built from [public source code](https://github.com/sriverogalan/better-screenshoot).

**Recommended:** Applications → right-click **Better Screenshoot** → **Open** → **Open**.

**If that does not work:**

1. Try again after copying the app from the `.dmg` to **Applications** (not running it from the disk image).
2. System Settings → **Privacy & Security** → scroll down → **Open Anyway**.
3. Terminal: `xattr -cr "/Applications/Better Screenshoot.app"` then right-click → **Open** again.

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
