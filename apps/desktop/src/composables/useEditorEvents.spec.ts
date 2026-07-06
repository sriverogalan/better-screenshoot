import { describe, it, expect, vi, beforeEach } from "vitest"
import { ref } from "vue"

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}))

import { listen } from "@tauri-apps/api/event"
import { useEditorEvents } from "./useEditorEvents"
import type { SavedCapture } from "../lib/tauri"

const mockListen = vi.mocked(listen)

describe("useEditorEvents", () => {
  beforeEach(() => {
    mockListen.mockReset()
  })

  it("setup registers window resize listener and two tauri event listeners", async () => {
    const addSpy = vi.spyOn(window, "addEventListener")
    const unlistenA = vi.fn()
    const unlistenB = vi.fn()
    mockListen
      .mockResolvedValueOnce(unlistenA as any)
      .mockResolvedValueOnce(unlistenB as any)

    const canvasEl = document.createElement("div")
    const canvasHost = ref<HTMLElement | null>(canvasEl)
    const measureHost = vi.fn()

    const events = useEditorEvents({
      canvasHost,
      measureHost,
      onCaptureComplete: vi.fn(),
      onEditorPresented: vi.fn(),
    })

    await events.setup()

    expect(addSpy).toHaveBeenCalledWith("resize", measureHost)
    expect(mockListen).toHaveBeenCalledTimes(2)
    expect(mockListen).toHaveBeenCalledWith("capture-complete", expect.any(Function))
    expect(mockListen).toHaveBeenCalledWith("editor-presented", expect.any(Function))

    addSpy.mockRestore()
  })

  it("cleanup removes all listeners without leaking", async () => {
    const removeSpy = vi.spyOn(window, "removeEventListener")
    const unlistenA = vi.fn()
    const unlistenB = vi.fn()
    mockListen
      .mockResolvedValueOnce(unlistenA as any)
      .mockResolvedValueOnce(unlistenB as any)

    const canvasHost = ref<HTMLElement | null>(null)
    const measureHost = vi.fn()

    const events = useEditorEvents({
      canvasHost,
      measureHost,
      onCaptureComplete: vi.fn(),
      onEditorPresented: vi.fn(),
    })

    await events.setup()
    events.cleanup()

    expect(removeSpy).toHaveBeenCalledWith("resize", measureHost)
    expect(unlistenA).toHaveBeenCalledOnce()
    expect(unlistenB).toHaveBeenCalledOnce()

    removeSpy.mockRestore()
  })

  it("cleanup is safe to call before setup completes", () => {
    const canvasHost = ref<HTMLElement | null>(null)
    const events = useEditorEvents({
      canvasHost,
      measureHost: vi.fn(),
      onCaptureComplete: vi.fn(),
      onEditorPresented: vi.fn(),
    })

    expect(() => events.cleanup()).not.toThrow()
  })

  it("calls onCaptureComplete with event payload when capture-complete fires", async () => {
    const onCaptureComplete = vi.fn()
    const unlistenA = vi.fn()
    const unlistenB = vi.fn()

    let captureHandler!: (event: { payload: SavedCapture }) => void
    mockListen.mockImplementation(async (eventName: string, handler: (event: any) => void) => {
      if (eventName === "capture-complete") {
        captureHandler = handler
        return unlistenA as any
      }
      return unlistenB as any
    })

    const canvasHost = ref<HTMLElement | null>(null)
    const events = useEditorEvents({
      canvasHost,
      measureHost: vi.fn(),
      onCaptureComplete,
      onEditorPresented: vi.fn(),
    })

    await events.setup()

    const mockPayload: SavedCapture = {
      id: "cap-1",
      file_path: "/test.png",
      data_url: "",
      width: 100,
      height: 100,
      created_at: "2026-01-01T00:00:00Z",
    }
    captureHandler({ payload: mockPayload })

    expect(onCaptureComplete).toHaveBeenCalledWith(mockPayload)
  })
})
