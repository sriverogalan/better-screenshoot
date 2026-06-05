import { getCurrentWindow, type Window } from "@tauri-apps/api/window";

const MAIN_WIDTH = 960;
const MAIN_HEIGHT = 640;
const SIZE_TOLERANCE = 8;
const CHECK_DEBOUNCE_MS = 250;

let guardPaused = false;

export function setMainWindowGuardPaused(paused: boolean): void {
  guardPaused = paused;
}

export async function hideMainIfInvalid(win: Window = getCurrentWindow()): Promise<void> {
  if (guardPaused || win.label !== "main" || !(await win.isVisible())) {
    return;
  }

  const [fullscreen, maximized, size, scale] = await Promise.all([
    win.isFullscreen(),
    win.isMaximized(),
    win.innerSize(),
    win.scaleFactor(),
  ]);

  const width = size.width / scale;
  const height = size.height / scale;
  const tooLarge =
    width > MAIN_WIDTH + SIZE_TOLERANCE || height > MAIN_HEIGHT + SIZE_TOLERANCE;

  if (!fullscreen && !maximized && !tooLarge) {
    return;
  }

  if (/Mac/i.test(navigator.userAgent)) {
    await win.setSimpleFullscreen(false);
  }
  await win.setFullscreen(false);
  await win.hide();
}

export function startMainWindowGuard(win: Window = getCurrentWindow()): () => void {
  if (win.label !== "main") {
    return () => {};
  }

  let debounceId: number | undefined;

  const scheduleCheck = () => {
    if (debounceId !== undefined) {
      clearTimeout(debounceId);
    }
    debounceId = window.setTimeout(() => {
      debounceId = undefined;
      void hideMainIfInvalid(win);
    }, CHECK_DEBOUNCE_MS);
  };

  void win.setMaximizable(false);
  void win.setResizable(false);

  let disposed = false;
  let unlistenResize: (() => void) | undefined;
  let unlistenFocus: (() => void) | undefined;

  void win.onResized(scheduleCheck).then((unlisten) => {
    if (disposed) {
      unlisten();
      return;
    }
    unlistenResize = unlisten;
  });

  void win.onFocusChanged(({ payload: focused }) => {
    if (focused) scheduleCheck();
  }).then((unlisten) => {
    if (disposed) {
      unlisten();
      return;
    }
    unlistenFocus = unlisten;
  });

  return () => {
    disposed = true;
    if (debounceId !== undefined) {
      clearTimeout(debounceId);
    }
    unlistenResize?.();
    unlistenFocus?.();
  };
}
