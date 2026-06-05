import { getCurrentWindow, type Window } from "@tauri-apps/api/window";

const MAC_NATIVE_FULLSCREEN_EXIT_DELAY_MS = 900;

export function isMacOsUserAgent(userAgent: string): boolean {
  return /Mac/i.test(userAgent);
}

async function resetEditorFullscreenState(win: Window): Promise<void> {
  if (isMacOsUserAgent(navigator.userAgent)) {
    await win.setSimpleFullscreen(false);

    if (await win.isFullscreen()) {
      await win.setFullscreen(false);
    }

    await new Promise((resolve) => {
      setTimeout(resolve, MAC_NATIVE_FULLSCREEN_EXIT_DELAY_MS);
    });
    return;
  }

  await win.setFullscreen(false);
}

export async function hideEditorWindow(
  win: Window = getCurrentWindow(),
): Promise<void> {
  if (win.label !== "editor") return;

  const wasFullscreen = await win.isFullscreen();

  if (wasFullscreen && isMacOsUserAgent(navigator.userAgent)) {
    await win.setSimpleFullscreen(false);
  } else if (wasFullscreen) {
    await win.setFullscreen(false);
  }

  await win.hide();

  if (!wasFullscreen) return;

  void resetEditorFullscreenState(win);
}
