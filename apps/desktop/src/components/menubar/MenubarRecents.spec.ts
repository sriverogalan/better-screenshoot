import { describe, it, expect, vi, beforeEach, afterEach } from "vitest"
import { mount } from "@vue/test-utils"
import { createPinia } from "pinia"
import { i18n } from "../../i18n/index"
import MenubarRecents from "./MenubarRecents.vue"

vi.mock("../../lib/tauri", () => ({
  getHistory: vi.fn(),
  openCaptureInEditor: vi.fn(),
  readCaptureDataUrl: vi.fn(async () => "data:image/png;base64,abc"),
  copyImageToClipboard: vi.fn(),
  deleteHistoryItem: vi.fn(),
}))

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: vi.fn((path: string) => `asset://${path}`),
}))

const hideMock = vi.fn()

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: vi.fn(() => ({
    hide: hideMock,
    onFocusChanged: vi.fn(async () => () => {}),
  })),
}))

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {}),
}))

vi.mock("@tauri-apps/plugin-opener", () => ({
  revealItemInDir: vi.fn(),
}))

vi.mock("@crabnebula/tauri-plugin-drag", () => ({
  startDrag: vi.fn(),
}))

import { getHistory, openCaptureInEditor, copyImageToClipboard, deleteHistoryItem } from "../../lib/tauri"
import { revealItemInDir } from "@tauri-apps/plugin-opener"
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

  it("opens a capture in the editor when its thumbnail is clicked", async () => {
    vi.mocked(getHistory).mockResolvedValue([makeRecord("a")])

    const wrapper = mount(MenubarRecents, {
      global: {
        plugins: [createPinia(), i18n],
      },
    })

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()

    await wrapper.find("button").trigger("click")
    await vi.runAllTimersAsync()

    expect(hideMock).toHaveBeenCalled()
    expect(openCaptureInEditor).toHaveBeenCalledWith("a")
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

  it("opens a context menu on right-click with copy, reveal, and delete actions", async () => {
    vi.mocked(getHistory).mockResolvedValue([makeRecord("a")])

    const wrapper = mount(MenubarRecents, {
      global: {
        plugins: [createPinia(), i18n],
      },
    })

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()

    await wrapper.find("button").trigger("contextmenu")
    await wrapper.vm.$nextTick()

    const menuButtons = wrapper.findAll("button")
    // thumbnail button + copy + reveal + delete
    expect(menuButtons.length).toBe(4)
  })

  it("copies the capture to the clipboard from the context menu", async () => {
    vi.mocked(getHistory).mockResolvedValue([makeRecord("a")])

    const wrapper = mount(MenubarRecents, {
      global: {
        plugins: [createPinia(), i18n],
      },
    })

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()
    await wrapper.find("button").trigger("contextmenu")
    await wrapper.vm.$nextTick()

    await wrapper.findAll("button").at(1)?.trigger("click")
    await vi.runAllTimersAsync()

    expect(copyImageToClipboard).toHaveBeenCalledWith("abc")
  })

  it("reveals the capture in Finder from the context menu", async () => {
    vi.mocked(getHistory).mockResolvedValue([makeRecord("a")])

    const wrapper = mount(MenubarRecents, {
      global: {
        plugins: [createPinia(), i18n],
      },
    })

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()
    await wrapper.find("button").trigger("contextmenu")
    await wrapper.vm.$nextTick()

    await wrapper.findAll("button").at(2)?.trigger("click")
    await vi.runAllTimersAsync()

    expect(revealItemInDir).toHaveBeenCalledWith("/tmp/a.png")
  })

  it("deletes the capture and refreshes the list from the context menu", async () => {
    vi.mocked(getHistory).mockResolvedValue([makeRecord("a")])

    const wrapper = mount(MenubarRecents, {
      global: {
        plugins: [createPinia(), i18n],
      },
    })

    await vi.runAllTimersAsync()
    await wrapper.vm.$nextTick()
    await wrapper.find("button").trigger("contextmenu")
    await wrapper.vm.$nextTick()

    await wrapper.findAll("button").at(3)?.trigger("click")
    await vi.runAllTimersAsync()

    expect(deleteHistoryItem).toHaveBeenCalledWith("a")
    expect(getHistory).toHaveBeenCalledTimes(2)
  })
})
