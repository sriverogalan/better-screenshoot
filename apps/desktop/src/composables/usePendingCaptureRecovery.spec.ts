import { describe, it, expect, vi, beforeEach } from "vitest"
import { setActivePinia, createPinia } from "pinia"
import { withSetup } from "../test-utils/with-setup"

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}))

vi.mock("../lib/tauri", () => ({
  peekPendingCapture: vi.fn(),
  openPendingCaptureInEditor: vi.fn(),
}))

vi.mock("../i18n/resolveError", () => ({
  translateAppError: vi.fn((_t: unknown, msg: string) => msg),
}))

import { listen } from "@tauri-apps/api/event"
import { peekPendingCapture } from "../lib/tauri"
import { usePendingCaptureRecovery } from "./usePendingCaptureRecovery"
import { i18n } from "../i18n/index"

const noopUnlisten = vi.fn()

describe("usePendingCaptureRecovery", () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    // listen returns a Promise<UnlistenFn>
    vi.mocked(listen).mockResolvedValue(noopUnlisten)
  })

  it("detects a pending capture on mount", async () => {
    const pendingCapture = {
      id: "cap1",
      file_path: "/tmp/cap.png",
      width: 1920,
      height: 1080,
      created_at: "2026-01-01T00:00:00Z",
      data_url: "data:image/png;base64,abc",
    }
    vi.mocked(peekPendingCapture).mockResolvedValue(pendingCapture)

    const pinia = createPinia()
    setActivePinia(pinia)

    const [result, cleanup] = withSetup(() => usePendingCaptureRecovery(), [pinia, i18n])

    // Wait for onMounted async work to complete
    await new Promise((r) => setTimeout(r, 0))

    expect(peekPendingCapture).toHaveBeenCalled()
    expect(result.pendingCapture.value).toEqual(pendingCapture)

    cleanup()
  })

  it("no-op when there is no pending capture", async () => {
    vi.mocked(peekPendingCapture).mockResolvedValue(null)

    const pinia = createPinia()
    setActivePinia(pinia)

    const [result, cleanup] = withSetup(() => usePendingCaptureRecovery(), [pinia, i18n])

    await new Promise((r) => setTimeout(r, 0))

    expect(result.pendingCapture.value).toBeNull()

    cleanup()
  })

  it("handles peekPendingCapture error gracefully", async () => {
    vi.mocked(peekPendingCapture).mockRejectedValue(new Error("IPC failure"))

    const pinia = createPinia()
    setActivePinia(pinia)

    const [result, cleanup] = withSetup(() => usePendingCaptureRecovery(), [pinia, i18n])

    await new Promise((r) => setTimeout(r, 0))

    // Should not throw — pendingCapture defaults to null on error
    expect(result.pendingCapture.value).toBeNull()

    cleanup()
  })
})
