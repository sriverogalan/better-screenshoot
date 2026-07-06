import { describe, it, expect, beforeEach, afterEach, vi } from "vitest"
import { setActivePinia, createPinia } from "pinia"
import { withSetup } from "../test-utils/with-setup"
import { useAppearance } from "./useAppearance"
import { useSettingsStore } from "../stores/settings"

vi.mock("../lib/tauri", () => ({
  getSettings: vi.fn(),
  updateSettings: vi.fn(),
}))

describe("useAppearance", () => {
  let cleanup: () => void

  beforeEach(() => {
    setActivePinia(createPinia())
    document.documentElement.className = ""
  })

  afterEach(() => {
    cleanup?.()
    document.documentElement.className = ""
  })

  it("applies theme-dark class when appearance is dark", () => {
    const pinia = createPinia()
    setActivePinia(pinia)

    const settingsStore = useSettingsStore()
    settingsStore.settings.appearance = "dark"

    ;[, cleanup] = withSetup(() => useAppearance(), [pinia])

    expect(document.documentElement.classList.contains("theme-dark")).toBe(true)
  })

  it("applies theme-light class when appearance is light", () => {
    const pinia = createPinia()
    setActivePinia(pinia)

    const settingsStore = useSettingsStore()
    settingsStore.settings.appearance = "light"

    ;[, cleanup] = withSetup(() => useAppearance(), [pinia])

    expect(document.documentElement.classList.contains("theme-light")).toBe(true)
  })

  it("applies theme-auto class when appearance is auto", () => {
    const pinia = createPinia()
    setActivePinia(pinia)

    const settingsStore = useSettingsStore()
    settingsStore.settings.appearance = "auto"

    ;[, cleanup] = withSetup(() => useAppearance(), [pinia])

    expect(document.documentElement.classList.contains("theme-auto")).toBe(true)
  })
})
