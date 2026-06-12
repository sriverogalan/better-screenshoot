import { beforeEach, describe, expect, it, vi } from "vitest";

const getVersionMock = vi.fn();
const checkMock = vi.fn();
const relaunchMock = vi.fn();
const downloadAndInstallMock = vi.fn();

vi.mock("@tauri-apps/api/app", () => ({
  getVersion: () => getVersionMock(),
}));

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: () => checkMock(),
}));

vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: () => relaunchMock(),
}));

describe("updater", () => {
  beforeEach(() => {
    getVersionMock.mockReset();
    checkMock.mockReset();
    relaunchMock.mockReset();
    downloadAndInstallMock.mockReset();
    vi.resetModules();
  });

  it("returns the app version", async () => {
    getVersionMock.mockResolvedValue("0.2.0");
    const { getAppVersion } = await import("./updater");

    await expect(getAppVersion()).resolves.toBe("0.2.0");
  });

  it("skips update checks outside production builds", async () => {
    const { checkForAppUpdate, isUpdaterSupported } = await import("./updater");

    expect(isUpdaterSupported()).toBe(false);
    await expect(checkForAppUpdate()).resolves.toBeNull();
    expect(checkMock).not.toHaveBeenCalled();
  });

  it("installs updates and relaunches", async () => {
    downloadAndInstallMock.mockResolvedValue(undefined);
    relaunchMock.mockResolvedValue(undefined);

    const update = {
      version: "0.3.0",
      body: "Bug fixes",
      date: "2026-06-12T10:00:00Z",
      downloadAndInstall: downloadAndInstallMock,
    };

    const { downloadAndInstallAppUpdate, summarizeUpdate } = await import("./updater");

    expect(summarizeUpdate(update as never)).toEqual({
      version: "0.3.0",
      notes: "Bug fixes",
      date: "2026-06-12T10:00:00Z",
    });

    await downloadAndInstallAppUpdate(update as never);

    expect(downloadAndInstallMock).toHaveBeenCalledOnce();
    expect(relaunchMock).toHaveBeenCalledOnce();
  });
});
