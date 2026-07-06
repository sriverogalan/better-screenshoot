import { describe, it, expect, vi, beforeEach, afterEach } from "vitest"
import { nextTick } from "vue"

vi.mock("vue-i18n", () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock("../lib/load-capture-image", () => ({
  loadCaptureImage: vi.fn(),
  disposeCaptureImage: vi.fn(),
}))

import { loadCaptureImage, disposeCaptureImage } from "../lib/load-capture-image"
import { useEditorLifecycle } from "./useEditorLifecycle"
import type { SavedCapture } from "../lib/tauri"

const mockLoadCaptureImage = vi.mocked(loadCaptureImage)
const mockDisposeCaptureImage = vi.mocked(disposeCaptureImage)

function makeCapture(overrides: Partial<SavedCapture> = {}): SavedCapture {
  return {
    id: "test-id",
    file_path: "/test/path.png",
    data_url: "",
    width: 1920,
    height: 1080,
    created_at: "2026-01-01T00:00:00Z",
    ...overrides,
  }
}

describe("useEditorLifecycle", () => {
  beforeEach(() => {
    vi.useFakeTimers()
    mockLoadCaptureImage.mockReset()
    mockDisposeCaptureImage.mockReset()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it("calls disposeCaptureImage and skips onLoaded when capture is null", async () => {
    const onLoaded = vi.fn()
    const lifecycle = useEditorLifecycle({ onLoaded })

    await lifecycle.loadCapture(null)

    expect(mockDisposeCaptureImage).toHaveBeenCalledOnce()
    expect(onLoaded).not.toHaveBeenCalled()
  })

  it("sets image refs on successful load and calls onLoaded", async () => {
    const mockElement = Object.assign(document.createElement("img"), {
      naturalWidth: 1920,
      naturalHeight: 1080,
    }) as HTMLImageElement
    mockLoadCaptureImage.mockResolvedValue({
      element: mockElement,
      dataUrl: "data:image/png;base64,abc",
    })

    const onLoaded = vi.fn()
    const lifecycle = useEditorLifecycle({ onLoaded })

    await lifecycle.loadCapture(makeCapture())

    expect(lifecycle.konvaImage.value).toBe(mockElement)
    expect(lifecycle.imagePreviewSrc.value).toBe("data:image/png;base64,abc")
    expect(lifecycle.imageNatural.value).toEqual({ width: 1920, height: 1080 })
    expect(onLoaded).toHaveBeenCalledOnce()
  })

  it("generation guard: second load wins when first is still pending", async () => {
    let resolveFirst!: (v: any) => void
    const firstPromise = new Promise<any>((resolve) => {
      resolveFirst = resolve
    })
    const secondElement = Object.assign(document.createElement("img"), {
      naturalWidth: 800,
      naturalHeight: 600,
    }) as HTMLImageElement

    mockLoadCaptureImage
      .mockReturnValueOnce(firstPromise)
      .mockResolvedValueOnce({
        element: secondElement,
        dataUrl: "data:image/png;base64,second",
      })

    const lifecycle = useEditorLifecycle({ onLoaded: vi.fn() })

    // Start first load — will hang
    const firstLoad = lifecycle.loadCapture(makeCapture())
    // Start second load — will resolve first
    const secondLoad = lifecycle.loadCapture(makeCapture())

    await secondLoad

    // Release the first load — must not overwrite second result
    resolveFirst({
      element: document.createElement("img"),
      dataUrl: "data:image/png;base64,first",
    })
    await firstLoad

    expect(lifecycle.konvaImage.value).toBe(secondElement)
    expect(lifecycle.imagePreviewSrc.value).toBe("data:image/png;base64,second")
  })

  it("sets imageLoadError after 8 s timeout when image never loads", async () => {
    // loadCaptureImage never resolves
    mockLoadCaptureImage.mockReturnValue(new Promise(() => {}))

    const lifecycle = useEditorLifecycle({ onLoaded: vi.fn() })

    void lifecycle.loadCapture(makeCapture())

    vi.advanceTimersByTime(8000)
    await nextTick()

    expect(lifecycle.imageLoadError.value).toBe("errors.imageDisplayFailed")
  })

  it("cleanup calls disposeCaptureImage", () => {
    const lifecycle = useEditorLifecycle({ onLoaded: vi.fn() })
    lifecycle.cleanup()
    expect(mockDisposeCaptureImage).toHaveBeenCalledOnce()
  })
})
