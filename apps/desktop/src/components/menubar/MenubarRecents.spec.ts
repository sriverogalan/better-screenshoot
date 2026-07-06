import { describe, it, expect, vi, beforeEach, afterEach } from "vitest"
import { mount } from "@vue/test-utils"
import { createPinia } from "pinia"
import { i18n } from "../../i18n/index"
import MenubarRecents from "./MenubarRecents.vue"

vi.mock("../../lib/tauri", () => ({
  getHistory: vi.fn(),
}))

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: vi.fn((path: string) => `asset://${path}`),
}))

import { getHistory } from "../../lib/tauri"
import type { CaptureRecord } from "@better-screenshoot/shared-types"

const makeRecord = (id: string): CaptureRecord => ({
  id,
  file_path: `/tmp/${id}.png`,
  width: 1920,
  height: 1080,
  created_at: "2026-01-01T00:00:00Z",
  tags: [],
})

describe("MenubarRecents", () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it("renders a grid of images when history returns records", async () => {
    vi.mocked(getHistory).mockResolvedValue([makeRecord("a"), makeRecord("b"), makeRecord("c")])

    const wrapper = mount(MenubarRecents, {
      global: {
        plugins: [createPinia(), i18n],
      },
    })

    // Wait for onMounted async fetch
    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()

    const imgs = wrapper.findAll("img")
    expect(imgs).toHaveLength(3)
  })

  it("shows empty state when history returns no records", async () => {
    vi.mocked(getHistory).mockResolvedValue([])

    const wrapper = mount(MenubarRecents, {
      global: {
        plugins: [createPinia(), i18n],
      },
    })

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()

    expect(wrapper.findAll("img")).toHaveLength(0)
    // Empty state paragraph should be visible
    const emptyP = wrapper.find("p[class*='text-fg-muted']")
    expect(emptyP.exists()).toBe(true)
  })

  it("shows empty state when getHistory rejects", async () => {
    vi.mocked(getHistory).mockRejectedValue(new Error("IPC error"))

    const wrapper = mount(MenubarRecents, {
      global: {
        plugins: [createPinia(), i18n],
      },
    })

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()

    expect(wrapper.findAll("img")).toHaveLength(0)
  })
})
