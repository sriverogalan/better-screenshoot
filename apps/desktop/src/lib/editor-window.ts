import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, type Window } from "@tauri-apps/api/window";

export function isMacOsUserAgent(userAgent: string): boolean {
  return /Mac/i.test(userAgent);
}

export function isCaptureSurfaceLabel(label: string): boolean {
  return label === "main" || label === "editor";
}

export async function hideEditorWindow(
  win: Window = getCurrentWindow(),
): Promise<void> {
  if (win.label !== "editor") return;

  const wasFullscreen = await win.isFullscreen();

  if (isMacOsUserAgent(navigator.userAgent)) {
    await win.setSimpleFullscreen(false);
    if (await win.isFullscreen()) {
      // Salir de la pantalla completa nativa de macOS antes de ocultar: si ocultamos
      // durante la animación queda un Space negro vacío.
      await win.setFullscreen(false);
      await new Promise((resolve) => setTimeout(resolve, 520));
    }
  } else if (wasFullscreen) {
    await win.setFullscreen(false);
  }

  try {
    await invoke("reset_editor_window_layout");
  } catch {
    // La ventana puede estar ya reseteándose.
  }

  await win.hide();
}

export async function exitCaptureEditor(
  win: Window = getCurrentWindow(),
): Promise<void> {
  if (win.label === "main") {
    const wasFullscreen = await win.isFullscreen();

    if (isMacOsUserAgent(navigator.userAgent)) {
      await win.setSimpleFullscreen(false);
      if (await win.isFullscreen()) {
        await win.setFullscreen(false);
        await new Promise((resolve) => setTimeout(resolve, 520));
      }
    } else if (wasFullscreen) {
      await win.setFullscreen(false);
    }

    try {
      await invoke("exit_main_editor_mode");
    } catch {
      // La ventana puede estar ya reseteándose.
    }

    await win.hide();
    return;
  }

  await hideEditorWindow(win);
}
