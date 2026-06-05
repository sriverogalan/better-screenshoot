import { describe, expect, it } from "vitest";
import { DEFAULT_HOTKEYS, DEFAULT_SETTINGS } from "./index";

describe("shared-types", () => {
  it("exposes sensible default hotkeys", () => {
    expect(DEFAULT_HOTKEYS.capture_area).toContain("Shift");
    expect(DEFAULT_SETTINGS.auto_copy).toBe(true);
  });
});
