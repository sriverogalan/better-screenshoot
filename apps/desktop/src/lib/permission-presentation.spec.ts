import { describe, expect, it } from "vitest"
import {
  deriveCapturePermissionPresentation,
} from "./permission-presentation"
import type { CaptureStatus } from "./tauri"

function status(overrides: Partial<CaptureStatus>): CaptureStatus {
  return {
    displays_found: 1,
    screen_capture_granted: true,
    messageCode: "displaysDetected",
    messageParams: { count: 1 },
    dev_binary_path: null,
    ...overrides,
  }
}

function expectPresentation(
  actual: ReturnType<typeof deriveCapturePermissionPresentation>,
  expected: ReturnType<typeof deriveCapturePermissionPresentation>,
) {
  expect(actual).toEqual(expected)
  expect(actual).not.toHaveProperty("impact")
}

describe("deriveCapturePermissionPresentation", () => {
  it("returns granted impact without repair actions when capture is available", () => {
    expectPresentation(
      deriveCapturePermissionPresentation(status({ screen_capture_granted: true })),
      {
        status: "granted",
        messageCode: "displaysDetected",
        messageParams: { count: 1 },
        impactCode: "settings.capturePermissionImpact.granted",
        devBinaryPath: null,
        showPermissionRequest: false,
        showRepairAction: false,
      },
    )
  })

  it("returns denied impact with permission request and dev binary path", () => {
    expectPresentation(
      deriveCapturePermissionPresentation(
        status({
          displays_found: 0,
          screen_capture_granted: false,
          messageCode: "macosPermissionRequired",
          messageParams: null,
          dev_binary_path: "/Applications/Better Screenshoot.app",
        }),
      ),
      {
        status: "denied",
        messageCode: "macosPermissionRequired",
        messageParams: null,
        impactCode: "settings.capturePermissionImpact.denied",
        devBinaryPath: "/Applications/Better Screenshoot.app",
        showPermissionRequest: true,
        showRepairAction: false,
      },
    )
  })

  it("returns unknown impact when no displays are detected on unsupported status", () => {
    expectPresentation(
      deriveCapturePermissionPresentation(
        status({
          displays_found: 0,
          screen_capture_granted: false,
          messageCode: "noDisplaysDetected",
          messageParams: null,
        }),
      ),
      {
        status: "unknown",
        messageCode: "noDisplaysDetected",
        messageParams: null,
        impactCode: "settings.capturePermissionImpact.unknown",
        devBinaryPath: null,
        showPermissionRequest: true,
        showRepairAction: false,
      },
    )
  })

  it("returns repair-needed impact when macOS reports granted permission but no displays", () => {
    expectPresentation(
      deriveCapturePermissionPresentation(
        status({
          displays_found: 0,
          screen_capture_granted: false,
          messageCode: "macosPermissionGrantedNoDisplays",
          messageParams: null,
        }),
      ),
      {
        status: "repair-needed",
        messageCode: "macosPermissionGrantedNoDisplays",
        messageParams: null,
        impactCode: "settings.capturePermissionImpact.repairNeeded",
        devBinaryPath: null,
        showPermissionRequest: false,
        showRepairAction: true,
      },
    )
  })
})
