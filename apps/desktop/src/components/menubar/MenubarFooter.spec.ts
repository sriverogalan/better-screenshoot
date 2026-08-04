import { describe, it, expect, vi } from "vitest"
// it.fails() marks this test as expected to fail — passes CI, fails if the bug gets accidentally "fixed"
import { mount } from "@vue/test-utils"
import { createPinia } from "pinia"
import { i18n } from "../../i18n/index"
import MenubarFooter from "./MenubarFooter.vue"

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: vi.fn(() => ({ hide: vi.fn() })),
}))

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: {
    getByLabel: vi.fn(() => null),
  },
}))

vi.mock("@tauri-apps/api/event", () => ({
  emitTo: vi.fn(),
}))

vi.mock("@tauri-apps/plugin-process", () => ({
  exit: vi.fn(),
}))

import { exit } from "@tauri-apps/plugin-process"

describe("MenubarFooter", () => {
  it("quits the app when the quit button is clicked", async () => {
    const wrapper = mount(MenubarFooter, {
      global: {
        plugins: [createPinia(), i18n],
      },
    })

    const quitButton = wrapper.findAll("button").at(-1)
    await quitButton?.trigger("click")

    expect(exit).toHaveBeenCalledWith(0)
  })

  it.fails(
    // intentional-fail: hardcoded hotkey — blocked on Slice C
    "does not render the hardcoded ⌘⇧H shortcut (should come from settings)",
    () => {
      const wrapper = mount(MenubarFooter, {
        global: {
          plugins: [createPinia(), i18n],
        },
      })

      // This test intentionally FAILS until Slice C replaces the hardcoded hotkey
      // with a dynamic value from settingsStore.settings.hotkeys.open_history.
      expect(wrapper.text()).not.toContain("⌘⇧H")
    },
  )
})
