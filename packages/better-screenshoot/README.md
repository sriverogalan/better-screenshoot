# Better Screenshoot

Control [Better Screenshoot](https://github.com/sriverogalan/better-screenshoot) from Raycast.

> **This extension does not include the app.** Raycast installs only these commands. You also need Better Screenshoot on your Mac (free download).

## Setup (end users)

### 1. Install this extension

From the Raycast Store, or via **Import Extension** during local development (see below).

### 2. Install Better Screenshoot

Download and install the desktop app from the [latest release](https://github.com/sriverogalan/better-screenshoot/releases/latest):

1. Open the `.dmg` and drag **Better Screenshoot** to **Applications**.
2. Open the app. If macOS blocks it, right-click → **Open** (unsigned build).
3. Grant **Screen Recording** (and **Accessibility** if prompted for global shortcuts).

### 3. Enable external control

In Better Screenshoot **Settings**, turn on **Allow external control** (Raycast, CLI, URL scheme).

Raycast commands will not work until steps 2 and 3 are done.

## Commands

| Command | Action |
| --- | --- |
| Capture Area | Select a region to capture |
| Capture Screen | Capture the full screen |
| Capture Window | Capture a specific window |
| Open History | Open capture history |

## Local development

Use **npm only** in this folder (not pnpm). The folder name must match `name` in `package.json` (`better-screenshoot`).

### First-time setup

1. Open **Raycast**.
2. Run **Import Extension**.
3. Select this folder:

```
.../better-screenshoot/packages/better-screenshoot
```

4. From the repo root:

```bash
pnpm raycast:dev
```

Or manually:

```bash
cd packages/better-screenshoot
npm install
npm run dev
```

Keep **Raycast open** while `npm run dev` runs. Commands appear in Raycast with a green dev indicator.

### If you see "Unable to install from local sources"

1. Run **Import Extension** again and pick `packages/better-screenshoot`.
2. Reinstall npm deps:

```bash
cd packages/better-screenshoot
rm -rf node_modules
npm install
npm run dev
```

Requires Better Screenshoot installed with external control enabled.

## Publish to Raycast Store

1. Confirm your Raycast account username matches `author` in `package.json` (`sriverogalan`).
2. Build and test the distribution bundle:

```bash
npm run build
npm run lint
```

3. Open the extension in Raycast and verify each command works.
4. Add store screenshots to `metadata/` (Raycast → Advanced → Window Capture, 2000×1250 PNG). At least three are recommended.
5. Publish (opens a PR in [raycast/extensions](https://github.com/raycast/extensions)):

```bash
npm run publish
```

Use `npm` in this folder for publishing — Raycast CI expects `package-lock.json`.
