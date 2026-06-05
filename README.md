# Better Screenshoot

Capturas de pantalla con editor integrado, atajos globales e historial. Disponible para macOS, Windows y Linux.

## Descargar

**[Última versión](https://github.com/sriverogalan/better-screenshoot/releases/latest)**

| Plataforma | Archivo | Requisitos |
|---|---|---|
| macOS (Apple Silicon) | `Better Screenshoot_*_aarch64.dmg` | macOS 12 o posterior |
| macOS (Intel) | `Better Screenshoot_*_x64.dmg` | macOS 12 o posterior |
| Windows | `*_x64-setup.exe` o `.msi` | Windows 10 o posterior |
| Linux | `.AppImage` o `.deb` | Ver [nota Wayland](#linux) |

## Instalación

### macOS

1. Descarga el `.dmg` que corresponda a tu Mac (Apple Silicon o Intel).
2. Abre el `.dmg` y arrastra **Better Screenshoot** a la carpeta **Aplicaciones**.
3. Abre la app. Si macOS muestra una advertencia de seguridad, consulta [Solución de problemas](#solución-de-problemas).
4. Concede el permiso de **Grabación de pantalla** cuando te lo pida (Ajustes del sistema → Privacidad y seguridad → Grabación de pantalla).
5. Para atajos globales fiables, activa también **Accesibilidad** si la app lo solicita.

### Windows

1. Descarga el instalador `.exe` o `.msi`.
2. Ejecuta el instalador y sigue los pasos.
3. Abre Better Screenshoot desde el menú Inicio. La app queda en la bandeja del sistema.

### Linux

1. Descarga el `.AppImage` o el paquete `.deb`.
2. **AppImage**: haz el archivo ejecutable (`chmod +x`) y ábrelo.
3. **Debian/Ubuntu**: instala con `sudo dpkg -i better-screenshoot_*.deb`.

> **Nota Wayland:** en entornos Wayland puede hacer falta el portal de escritorio (`xdg-desktop-portal`). Consulta [docs/api.md](docs/api.md#linux) para más detalle.

## Primeros pasos

Better Screenshoot vive en la **bandeja del sistema**. Desde ahí puedes capturar, abrir el historial o ir a ajustes.

Tras capturar, se abre el **editor** para anotar la imagen: flechas, rectángulos, texto, resaltado, trazo libre y desenfoque. Al terminar, la captura se copia al portapapeles y puedes guardarla en disco.

### Atajos por defecto

| Acción | macOS | Windows / Linux |
|---|---|---|
| Capturar área | ⌘⇧X | Ctrl+⇧X |
| Capturar pantalla | ⌘⇧⌥S | Ctrl+⇧Alt+S |
| Capturar ventana | ⌘⇧⌥W | Ctrl+⇧Alt+W |
| Abrir historial | ⌘⇧H | Ctrl+⇧H |

Puedes cambiar los atajos en **Ajustes** dentro de la app.

## Solución de problemas

### macOS no deja abrir la app

Las versiones actuales no están firmadas con certificado de Apple. Es normal ver el aviso *“no se puede abrir porque proviene de un desarrollador no identificado”*.

**Opción A:** clic derecho sobre la app → **Abrir** → confirmar.

**Opción B:** Ajustes del sistema → **Privacidad y seguridad** → botón **Abrir de todos modos**.

### La captura no funciona en macOS

Ve a **Ajustes del sistema → Privacidad y seguridad → Grabación de pantalla** y activa Better Screenshoot. Reinicia la app si ya la tenías abierta.

### Los atajos globales no responden

En macOS, añade Better Screenshoot en **Ajustes → Privacidad y seguridad → Accesibilidad**.

## Publicar una nueva versión

Para mantenedores del repositorio:

```bash
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions creará un **borrador de release** con los instaladores (.dmg, .exe, .deb, etc.). Revísalo y publícalo cuando esté listo.

Ver [docs/branching.md](docs/branching.md) para el flujo de ramas y protección de `main`.

---

## Para desarrolladores

### Stack

- **Tauri 2** + **Vue 3** + **TypeScript** + **Tailwind CSS**
- **Rust** (`capture-core`) para captura nativa
- **Pinia** para estado, **SQLite** para historial

### Desarrollo local

```bash
pnpm install
pnpm dev
```

Compilar instaladores:

```bash
pnpm build          # todas las plataformas soportadas en tu OS
pnpm build:mac      # solo .dmg en macOS
```

El `.dmg` se genera en `apps/desktop/src-tauri/target/release/bundle/dmg/`.

### Estructura

```
apps/desktop/          # App Tauri + Vue
packages/capture-core/ # Motor de captura Rust
packages/shared-types/ # Tipos IPC compartidos
packages/licensing/    # Tiers open core + validación de licencias
packages/raycast-extension/ # Extensión Raycast (macOS)
cli/                   # CLI better-screenshoot
docs/api.md            # URL scheme y CLI
```

### Integraciones

```bash
open "betterscreenshoot://capture-area"
better-screenshoot-cli open capture-screen
```

Ver [docs/api.md](docs/api.md).

### Ramas y releases

Flujo de trabajo, protección de `main` y versionado: [docs/branching.md](docs/branching.md).

## Licencia

Open core — Community (OSS) + tiers Pro/Cloud/Team de pago.
