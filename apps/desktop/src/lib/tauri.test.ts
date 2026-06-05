import { beforeEach, describe, expect, it, vi } from "vitest";
import { DEFAULT_HOTKEYS } from "@better-screenshoot/shared-types";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  convertFileSrc: (path: string) => path,
}));

describe("tauri integration contracts", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("uses shared default hotkeys", () => {
    expect(DEFAULT_HOTKEYS.capture_area).toContain("Shift");
  });

  it("openCaptureInEditor invokes backend command", async () => {
    invokeMock.mockResolvedValue(undefined);
    const { openCaptureInEditor } = await import("./tauri");

    await openCaptureInEditor("capture-123");

    expect(invokeMock).toHaveBeenCalledWith("open_capture_in_editor", {
      captureId: "capture-123",
    });
  });

  it("discardCapture invokes backend command", async () => {
    invokeMock.mockResolvedValue(undefined);
    const { discardCapture } = await import("./tauri");

    await discardCapture("capture-123", "/tmp/capture.png");

    expect(invokeMock).toHaveBeenCalledWith("discard_capture", {
      captureId: "capture-123",
      filePath: "/tmp/capture.png",
    });
  });
});
