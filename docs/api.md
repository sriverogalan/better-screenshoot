# Better Screenshoot — Integration API

## URL Scheme

Registered: `betterscreenshoot://`

Requires **Allow external control** to be enabled in Settings.

| URL | Action |
|-----|--------|
| `betterscreenshoot://capture-area` | Opens the region selector |
| `betterscreenshoot://capture-screen` | Captures the primary screen |
| `betterscreenshoot://capture-window` | Opens the window selector |
| `betterscreenshoot://open-history` | Opens history |
| `betterscreenshoot://open-settings` | Opens settings |

### Example

```bash
open "betterscreenshoot://capture-area"
```

## CLI

Binary: `better-screenshoot`

```bash
# Trigger action via app (GUI)
better-screenshoot-cli open capture-area

# Headless capture
better-screenshoot-cli capture --output ~/Desktop/shot.png screen
better-screenshoot-cli capture --output shot.png --json window 12345
better-screenshoot-cli capture --output region.png region --display 0 --x 100 --y 100 --width 400 --height 300

# List displays and windows
better-screenshoot displays
better-screenshoot windows
```

## Raycast

See the extension in `packages/raycast-extension/`. It uses the URL scheme internally.

## Permissions (macOS)

- **Screen Recording** — required for captures
- **Accessibility** — recommended for reliable global shortcuts
