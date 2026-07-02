## Get started (macOS)

Full guide: https://github.com/sriverogalan/better-screenshoot#get-started

This build is **not** signed with an Apple Developer certificate. Install via Homebrew to avoid macOS security warnings automatically.

### Option 1: Homebrew (recommended)

```sh
brew install --cask sriverogalan/better-screenshoot/better-screenshoot
```

Open **Better Screenshoot** from Launchpad or Spotlight.
Allow **Screen Recording** (and **Accessibility** for global shortcuts) when prompted.

### Option 2: Direct download (DMG)

1. Download the `.dmg` for your Mac — Apple Silicon (`*_aarch64.dmg`) or Intel (`*_x64.dmg`).
2. Open the `.dmg` and drag **Better Screenshoot** to **Applications**.
3. If macOS says **"Better Screenshoot is damaged and can't be opened"**, run once in Terminal:

   ```sh
   xattr -cr "/Applications/Better Screenshoot.app"
   ```

4. Allow **Screen Recording** (and **Accessibility** for global shortcuts) when prompted.

### Instalación (español)

**Opción 1 — Homebrew (recomendado):**

```sh
brew install --cask sriverogalan/better-screenshoot/better-screenshoot
```

**Opción 2 — Descarga directa:**

1. Descarga el `.dmg` — Apple Silicon (`*_aarch64.dmg`) o Intel (`*_x64.dmg`).
2. Abre el `.dmg` y arrastra **Better Screenshoot** a **Aplicaciones**.
3. Si macOS muestra **"Better Screenshoot está dañado y no se puede abrir"**, ejecuta una vez en Terminal:

   ```sh
   xattr -cr "/Applications/Better Screenshoot.app"
   ```

4. Acepta los permisos de **Grabación de pantalla** y **Accesibilidad** cuando se soliciten.

---

Installed apps pick up updater releases from `latest.json` once this release is published.
