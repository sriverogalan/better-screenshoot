import { describe, expect, it } from "vitest";
import { DEFAULT_HOTKEYS, DEFAULT_SETTINGS, SYSTEM_REPLACEMENT_HOTKEYS } from "./index";

describe("shared-types", () => {
  it("exposes sensible default hotkeys", () => {
    expect(DEFAULT_HOTKEYS.capture_area).toContain("Shift");
    expect(DEFAULT_SETTINGS.auto_copy).toBe(true);
  });

  it("maps system replacement hotkeys to macOS defaults", () => {
    expect(SYSTEM_REPLACEMENT_HOTKEYS.capture_screen).toBe("Command+Shift+3");
    expect(SYSTEM_REPLACEMENT_HOTKEYS.capture_area).toBe("Command+Shift+4");
    expect(SYSTEM_REPLACEMENT_HOTKEYS.capture_window).toBe("Command+Shift+5");
  });

  it("defaults to independent system capture mode", () => {
    expect(DEFAULT_SETTINGS.system_capture_mode).toBe("independent");
  });
});
