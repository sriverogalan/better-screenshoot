import { describe, it, expect, vi, beforeEach, afterEach } from "vitest"
import { mount, flushPromises, type VueWrapper } from "@vue/test-utils"
import { createPinia } from "pinia"
import { i18n } from "../i18n/index"
import OnboardingView from "./OnboardingView.vue"
import type { CaptureStatus } from "../lib/tauri"

vi.mock("vue-router", () => ({
  useRouter: () => ({
    replace: vi.fn(),
  }),
}))

vi.mock("../lib/tauri", () => ({
  getCaptureStatus: vi.fn(),
  openScreenRecordingSettings: vi.fn(),
  requestScreenCapturePermission: vi.fn(),
  setLaunchAtLogin: vi.fn(),
  getSettings: vi.fn(),
  updateSettings: vi.fn(),
}))

import {
  getCaptureStatus,
  openScreenRecordingSettings,
  requestScreenCapturePermission,
  updateSettings,
} from "../lib/tauri"

const captureStatus = (messageCode: string): CaptureStatus => ({
  displays_found: 0,
  screen_capture_granted: false,
  messageCode,
  dev_binary_path: null,
})

function findOpenSettingsButton(wrapper: VueWrapper) {
  const button = wrapper
    .findAll("button")
    .find((candidate) => candidate.text() === "Open System Settings")

  expect(button).toBeDefined()
  return button!
}

async function mountAtPermissionStep() {
  vi.mocked(getCaptureStatus).mockResolvedValue(
    captureStatus("macosPermissionRequired"),
  )

  const wrapper = mount(OnboardingView, {
    global: {
      plugins: [createPinia(), i18n],
    },
  })

  await wrapper.get("button").trigger("click")
  await flushPromises()

  vi.mocked(getCaptureStatus).mockClear()
  return wrapper
}

describe("OnboardingView", () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.clearAllMocks()
    vi.mocked(updateSettings).mockImplementation(async (settings) => settings)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it("opens screen recording settings before continuing status checks and polling", async () => {
    vi.mocked(requestScreenCapturePermission).mockResolvedValue(true)
    vi.mocked(openScreenRecordingSettings).mockResolvedValue()
    const wrapper = await mountAtPermissionStep()

    await findOpenSettingsButton(wrapper).trigger("click")
    await flushPromises()

    expect(requestScreenCapturePermission).toHaveBeenCalledTimes(1)
    expect(openScreenRecordingSettings).toHaveBeenCalledTimes(1)
    expect(getCaptureStatus).toHaveBeenCalledTimes(1)

    await vi.advanceTimersByTimeAsync(2000)

    expect(getCaptureStatus).toHaveBeenCalledTimes(2)

    wrapper.unmount()
  })

  it("opens settings and continues status checks when requesting screen capture permission rejects", async () => {
    vi.mocked(requestScreenCapturePermission).mockRejectedValue(
      new Error("permission prompt failed"),
    )
    vi.mocked(openScreenRecordingSettings).mockResolvedValue()
    const wrapper = await mountAtPermissionStep()

    await findOpenSettingsButton(wrapper).trigger("click")
    await flushPromises()

    expect(requestScreenCapturePermission).toHaveBeenCalledTimes(1)
    expect(openScreenRecordingSettings).toHaveBeenCalledTimes(1)
    expect(getCaptureStatus).toHaveBeenCalledTimes(1)

    await vi.advanceTimersByTimeAsync(2000)

    expect(getCaptureStatus).toHaveBeenCalledTimes(2)

    wrapper.unmount()
  })

  it("continues status checks and polling when opening screen recording settings rejects", async () => {
    vi.mocked(requestScreenCapturePermission).mockResolvedValue(false)
    vi.mocked(openScreenRecordingSettings).mockRejectedValue(
      new Error("settings failed"),
    )
    const wrapper = await mountAtPermissionStep()

    await findOpenSettingsButton(wrapper).trigger("click")
    await flushPromises()

    expect(requestScreenCapturePermission).toHaveBeenCalledTimes(1)
    expect(openScreenRecordingSettings).toHaveBeenCalledTimes(1)
    expect(getCaptureStatus).toHaveBeenCalledTimes(1)

    await vi.advanceTimersByTimeAsync(2000)

    expect(getCaptureStatus).toHaveBeenCalledTimes(2)

    wrapper.unmount()
  })
})
