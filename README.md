# Better Screenshoot

Herramienta de capturas multiplataforma orientada a desarrolladores. macOS, Linux y Windows.

## Stack

- **Tauri 2** + **Vue 3** + **TypeScript** + **Tailwind CSS**
- **Rust** (`capture-core`) para captura nativa vía `xcap`
- **Pinia** para estado, **SQLite** para historial

## Desarrollo

```bash
pnpm install
pnpm dev
```

## Estructura

```
apps/desktop/          # App Tauri + Vue
packages/capture-core/ # Motor de captura Rust
packages/shared-types/ # Tipos IPC compartidos
packages/licensing/    # Tiers open core + validación de licencias
packages/raycast-extension/ # Extensión Raycast (macOS)
cli/                   # CLI better-screenshoot
docs/api.md            # URL scheme y CLI
```

## Integraciones

```bash
open "betterscreenshoot://capture-area"
better-screenshoot-cli open capture-screen
```

Ver [docs/api.md](docs/api.md).

## Licencia

Open core — Community (OSS) + tiers Pro/Cloud/Team de pago.
