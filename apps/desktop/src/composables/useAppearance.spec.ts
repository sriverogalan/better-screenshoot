import { readFileSync } from "node:fs"
import { fileURLToPath } from "node:url"
import { dirname, resolve } from "node:path"
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

  it("replaces the previous theme class when appearance changes", () => {
    const pinia = createPinia()
    setActivePinia(pinia)

    const settingsStore = useSettingsStore()
    settingsStore.settings.appearance = "dark"

    const [appearance, dispose] = withSetup(() => useAppearance(), [pinia])
    cleanup = dispose

    appearance.applyAppearance("light")

    expect([...document.documentElement.classList]).toEqual(["theme-light"])
  })

  it("defines visible app-wide tokens for light, dark, and auto modes", () => {
    const currentDir = dirname(fileURLToPath(import.meta.url))
    const styles = readFileSync(resolve(currentDir, "../styles.css"), "utf8")

    expect(styles).toContain("html.theme-light")
    expect(styles).toContain("html.theme-dark")
    expect(styles).toContain("html.theme-auto")
    expect(styles).toContain("@media (prefers-color-scheme: light)")
    expect(styles).toContain("color-scheme: light")
    expect(styles).toContain("color-scheme: dark")
  })
})
