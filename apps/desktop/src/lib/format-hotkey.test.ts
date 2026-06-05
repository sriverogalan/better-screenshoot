import { describe, expect, it } from "vitest";
import { formatHotkey } from "./format-hotkey";

describe("formatHotkey", () => {
  it("formats macOS modifiers with symbols", () => {
    expect(formatHotkey("CommandOrControl+Shift+X")).toBe("⌘⇧X");
    expect(formatHotkey("CommandOrControl+Shift+Option+S")).toBe("⌘⇧⌥S");
    expect(formatHotkey("CommandOrControl+Shift+Option+W")).toBe("⌘⇧⌥W");
  });

  it("leaves unknown keys unchanged", () => {
    expect(formatHotkey("F12")).toBe("F12");
  });
});
