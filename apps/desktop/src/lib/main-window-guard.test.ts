import { describe, expect, it, vi } from "vitest";
import { hideMainIfInvalid, setMainWindowGuardPaused } from "./main-window-guard";

function createMockWindow(overrides: Partial<{
  label: string;
  visible: boolean;
  fullscreen: boolean;
  maximized: boolean;
  width: number;
  height: number;
  scale: number;
}>) {
  const hide = vi.fn();
  const setFullscreen = vi.fn();
  const setSimpleFullscreen = vi.fn();

  const win = {
    label: overrides.label ?? "main",
    isVisible: vi.fn(async () => overrides.visible ?? true),
    isFullscreen: vi.fn(async () => overrides.fullscreen ?? false),
    isMaximized: vi.fn(async () => overrides.maximized ?? false),
    innerSize: vi.fn(async () => ({
      width: overrides.width ?? 960,
      height: overrides.height ?? 640,
    })),
    scaleFactor: vi.fn(async () => overrides.scale ?? 1),
    hide,
    setFullscreen,
    setSimpleFullscreen,
  };

  return { win, hide, setFullscreen, setSimpleFullscreen };
}

describe("hideMainIfInvalid", () => {
  it("oculta el hub en fullscreen", async () => {
    const { win, hide } = createMockWindow({ fullscreen: true });
    await hideMainIfInvalid(win as never);
    expect(hide).toHaveBeenCalledOnce();
  });

  it("oculta el hub si supera el tamaño permitido", async () => {
    const { win, hide } = createMockWindow({ width: 1400, height: 900 });
    await hideMainIfInvalid(win as never);
    expect(hide).toHaveBeenCalledOnce();
  });

  it("no oculta el hub con tamaño válido", async () => {
    const { win, hide } = createMockWindow({});
    await hideMainIfInvalid(win as never);
    expect(hide).not.toHaveBeenCalled();
  });

  it("no oculta el hub mientras la captura está activa", async () => {
    const { win, hide } = createMockWindow({ fullscreen: true });
    setMainWindowGuardPaused(true);
    await hideMainIfInvalid(win as never);
    expect(hide).not.toHaveBeenCalled();
    setMainWindowGuardPaused(false);
  });

  it("ignora otras ventanas", async () => {
    const { win, hide } = createMockWindow({
      label: "editor",
      fullscreen: true,
    });
    await hideMainIfInvalid(win as never);
    expect(hide).not.toHaveBeenCalled();
  });
});
