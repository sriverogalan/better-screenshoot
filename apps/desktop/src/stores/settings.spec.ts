import { describe, it, expect, vi, beforeEach } from "vitest"
import { setActivePinia, createPinia } from "pinia"
import { DEFAULT_SETTINGS } from "@better-screenshoot/shared-types"

vi.mock("../lib/tauri", () => ({
  getSettings: vi.fn(),
  updateSettings: vi.fn(),
}))

import { getSettings, updateSettings } from "../lib/tauri"
import { useSettingsStore } from "./settings"

describe("useSettingsStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it("merges partial response with defaults on load", async () => {
    const partial = { ...DEFAULT_SETTINGS, appearance: "dark" as const }
    vi.mocked(getSettings).mockResolvedValue(partial)

    const store = useSettingsStore()
    await store.load()

    expect(store.settings.appearance).toBe("dark")
    expect(store.settings.auto_copy).toBe(DEFAULT_SETTINGS.auto_copy)
    expect(store.loading).toBe(false)
  })

  it("retains DEFAULT_SETTINGS when load fails and does not rethrow", async () => {
    vi.mocked(getSettings).mockRejectedValue(new Error("IPC failure"))

    const store = useSettingsStore()
    await expect(store.load()).resolves.toBeUndefined()

    expect(store.settings).toEqual(DEFAULT_SETTINGS)
    expect(store.loading).toBe(false)
  })

  it("calls updateSettings with the correct payload on save", async () => {
    const updated = { ...DEFAULT_SETTINGS, auto_copy: false }
    vi.mocked(updateSettings).mockResolvedValue(updated)

    const store = useSettingsStore()
    await store.save(updated)

    expect(updateSettings).toHaveBeenCalledWith(updated)
    expect(store.settings.auto_copy).toBe(false)
  })
})
