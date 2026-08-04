import { describe, it, expect, beforeEach } from "vitest"
import { setActivePinia, createPinia } from "pinia"
import { useCaptureStore } from "./capture"
import type { SavedCapture } from "../lib/tauri"

const makeSavedCapture = (overrides: Partial<SavedCapture> = {}): SavedCapture => ({
  id: "test-id",
  file_path: "/tmp/cap.png",
  width: 1920,
  height: 1080,
  created_at: "2026-01-01T00:00:00Z",
  data_url: "data:image/png;base64,abc123",
  ...overrides,
})

describe("useCaptureStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it("starts with null current and editorBase64", () => {
    const store = useCaptureStore()
    expect(store.current).toBeNull()
    expect(store.editorBase64).toBeNull()
  })

  it("setCapture stores the capture and strips data URL prefix from editorBase64", () => {
    const store = useCaptureStore()
    const capture = makeSavedCapture({ data_url: "data:image/png;base64,abc123" })
    store.setCapture(capture)

    expect(store.current).toEqual(capture)
    expect(store.editorBase64).toBe("abc123")
  })

  it("setCapture sets editorBase64 to null when data_url is absent", () => {
    const store = useCaptureStore()
    const capture = makeSavedCapture({ data_url: undefined })
    store.setCapture(capture)

    expect(store.editorBase64).toBeNull()
  })

  it("clear resets both current and editorBase64 to null", () => {
    const store = useCaptureStore()
    store.setCapture(makeSavedCapture())
    store.clear()

    expect(store.current).toBeNull()
    expect(store.editorBase64).toBeNull()
  })
})
