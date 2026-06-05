import { describe, expect, it } from "vitest";
import { isCaptureSessionReady } from "./capture-session";

describe("capture editor session", () => {
  it("is ready only when the same capture has preview and history baseline", () => {
    expect(
      isCaptureSessionReady({
        currentCaptureId: "capture-1",
        incomingCaptureId: "capture-1",
        imagePreviewReady: true,
        historyEntries: 1,
      }),
    ).toBe(true);
  });

  it("requires a fresh session for a different capture", () => {
    expect(
      isCaptureSessionReady({
        currentCaptureId: "capture-1",
        incomingCaptureId: "capture-2",
        imagePreviewReady: true,
        historyEntries: 1,
      }),
    ).toBe(false);
  });

  it("requires preparation when the image preview is not ready", () => {
    expect(
      isCaptureSessionReady({
        currentCaptureId: "capture-1",
        incomingCaptureId: "capture-1",
        imagePreviewReady: false,
        historyEntries: 1,
      }),
    ).toBe(false);
  });

  it("requires preparation when the editable history baseline is missing", () => {
    expect(
      isCaptureSessionReady({
        currentCaptureId: "capture-1",
        incomingCaptureId: "capture-1",
        imagePreviewReady: true,
        historyEntries: 0,
      }),
    ).toBe(false);
  });
});
