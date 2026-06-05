import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: vi.fn((path: string) => `asset://${path}`),
  invoke: vi.fn(),
}));

import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import {
  disposeCaptureImage,
  loadCaptureImage,
  loadHtmlImage,
  resolveCaptureDataUrl,
} from "./load-capture-image";
import type { SavedCapture } from "./tauri";

function createCapture(overrides: Partial<SavedCapture> = {}): SavedCapture {
  return {
    id: "test-id",
    file_path: "",
    width: 100,
    height: 80,
    created_at: "2026-01-01T00:00:00Z",
    data_url: "",
    ...overrides,
  };
}

function mockImageConstructor(options?: { failOn?: string[] }) {
  vi.stubGlobal("Image", function ImageMock() {
    const img = {
      onload: null as (() => void) | null,
      onerror: null as (() => void) | null,
      _src: "",
      naturalWidth: 100,
      naturalHeight: 80,
      set src(value: string) {
        this._src = value;
        if (options?.failOn?.includes(value)) {
          this.onerror?.();
          return;
        }
        this.onload?.();
      },
      get src() {
        return this._src;
      },
    };
    return img as unknown as HTMLImageElement;
  });
}

describe("resolveCaptureDataUrl", () => {
  afterEach(() => {
    vi.clearAllMocks();
    vi.unstubAllGlobals();
  });

  it("returns data_url when present", async () => {
    const capture = createCapture({ data_url: "data:image/png;base64,AAAA" });
    await expect(resolveCaptureDataUrl(capture)).resolves.toBe(
      "data:image/png;base64,AAAA",
    );
    expect(invoke).not.toHaveBeenCalled();
    expect(convertFileSrc).not.toHaveBeenCalled();
  });

  it("uses convertFileSrc for file_path", async () => {
    const capture = createCapture({ file_path: "/tmp/capture.png" });

    await expect(resolveCaptureDataUrl(capture)).resolves.toBe(
      "asset:///tmp/capture.png",
    );
    expect(convertFileSrc).toHaveBeenCalledWith("/tmp/capture.png");
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("loadCaptureImage", () => {
  afterEach(() => {
    disposeCaptureImage();
    vi.clearAllMocks();
    vi.unstubAllGlobals();
  });

  it("loads image element via data_url", async () => {
    mockImageConstructor();
    const capture = createCapture({ data_url: "data:image/png;base64,AAAA" });
    const loaded = await loadCaptureImage(capture);

    expect(loaded.dataUrl).toBe("data:image/png;base64,AAAA");
    expect(loaded.element.naturalWidth).toBe(100);
    expect(invoke).not.toHaveBeenCalled();
  });

  it("loads image element via convertFileSrc", async () => {
    mockImageConstructor();
    const capture = createCapture({ file_path: "/tmp/capture.png" });
    const loaded = await loadCaptureImage(capture);

    expect(loaded.dataUrl).toBe("asset:///tmp/capture.png");
    expect(loaded.element.naturalWidth).toBe(100);
    expect(convertFileSrc).toHaveBeenCalledWith("/tmp/capture.png");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("falls back to read_capture_data_url when asset URL fails", async () => {
    mockImageConstructor({ failOn: ["asset:///tmp/capture.png"] });
    vi.mocked(invoke).mockResolvedValue("data:image/png;base64,BBBB");
    const capture = createCapture({ file_path: "/tmp/capture.png" });

    const loaded = await loadCaptureImage(capture);

    expect(loaded.dataUrl).toBe("data:image/png;base64,BBBB");
    expect(invoke).toHaveBeenCalledWith("read_capture_data_url", {
      filePath: "/tmp/capture.png",
    });
  });

  it("rejects when capture has no image source", async () => {
    await expect(loadCaptureImage(createCapture())).rejects.toThrow(
      "La captura no tiene imagen",
    );
  });
});

describe("loadHtmlImage", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("sets crossOrigin before loading src", async () => {
    let crossOrigin = "";
    let srcAssigned = false;

    vi.stubGlobal("Image", function ImageMock() {
      const img = {
        onload: null as (() => void) | null,
        onerror: null as (() => void) | null,
        _src: "",
        set crossOrigin(value: string) {
          if (srcAssigned) {
            throw new Error("crossOrigin must be set before src");
          }
          crossOrigin = value;
        },
        get crossOrigin() {
          return crossOrigin;
        },
        set src(value: string) {
          srcAssigned = true;
          this._src = value;
          this.onload?.();
        },
        get src() {
          return this._src;
        },
      };
      return img as unknown as HTMLImageElement;
    });

    await loadHtmlImage("data:image/png;base64,AAAA");
    expect(crossOrigin).toBe("anonymous");
  });

  it("rejects when image decoding fails", async () => {
    vi.stubGlobal("Image", function ImageMock() {
      const img = {
        onload: null as (() => void) | null,
        onerror: null as (() => void) | null,
        _src: "",
        set src(_value: string) {
          this.onerror?.();
        },
        get src() {
          return this._src;
        },
      };
      return img as unknown as HTMLImageElement;
    });

    await expect(loadHtmlImage("data:image/png;base64,broken")).rejects.toThrow(
      "Error al decodificar la imagen",
    );
  });
});
