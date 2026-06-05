# Better Screenshoot — API de integración

## URL Scheme

Registro: `betterscreenshoot://`

Requiere que **Permitir control externo** esté activado en Ajustes.

| URL | Acción |
|-----|--------|
| `betterscreenshoot://capture-area` | Abre el selector de región |
| `betterscreenshoot://capture-screen` | Captura la pantalla principal |
| `betterscreenshoot://capture-window` | Abre el selector de ventanas |
| `betterscreenshoot://open-history` | Abre el historial |
| `betterscreenshoot://open-settings` | Abre ajustes |

### Ejemplo (macOS / Linux)

```bash
open "betterscreenshoot://capture-area"
```

### Ejemplo (Windows)

```powershell
start betterscreenshoot://capture-area
```

## CLI

Binario: `better-screenshoot`

```bash
# Disparar acción vía app (GUI)
better-screenshoot-cli open capture-area

# Captura headless
better-screenshoot-cli capture --output ~/Desktop/shot.png screen
better-screenshoot-cli capture --output shot.png --json window 12345
better-screenshoot-cli capture --output region.png region --display 0 --x 100 --y 100 --width 400 --height 300

# Listar displays y ventanas
better-screenshoot displays
better-screenshoot windows
```

## Raycast

Ver extensión en `packages/raycast-extension/`. Usa el URL scheme internamente.

## Permisos por plataforma

### macOS
- **Grabación de pantalla** — obligatorio para capturas
- **Accesibilidad** — recomendado para atajos globales fiables

### Windows
- Sin permisos especiales en la mayoría de configuraciones

### Linux
- **X11**: captura directa vía xcap
- **Wayland**: requiere `xdg-desktop-portal` y backend del compositor; usar `capture via portal` o Flatpak
