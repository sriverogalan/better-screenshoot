import { describe, expect, it } from "vitest";
import { parseHotkeyEvent } from "./parse-hotkey-event";

function keyEvent(partial: Partial<KeyboardEvent> & { key: string; code: string }): KeyboardEvent {
  return {
    key: partial.key,
    code: partial.code,
    metaKey: partial.metaKey ?? false,
    ctrlKey: partial.ctrlKey ?? false,
    shiftKey: partial.shiftKey ?? false,
    altKey: partial.altKey ?? false,
  } as KeyboardEvent;
}

describe("parseHotkeyEvent", () => {
  it("builds CommandOrControl chords from meta/ctrl", () => {
    expect(
      parseHotkeyEvent(
        keyEvent({ key: "x", code: "KeyX", metaKey: true, shiftKey: true }),
      ),
    ).toBe("CommandOrControl+Shift+X");

    expect(
      parseHotkeyEvent(
        keyEvent({ key: "s", code: "KeyS", ctrlKey: true, shiftKey: true, altKey: true }),
      ),
    ).toBe("CommandOrControl+Shift+Option+S");
  });

  it("returns null for Escape and modifier-only presses", () => {
    expect(parseHotkeyEvent(keyEvent({ key: "Escape", code: "Escape" }))).toBeNull();
    expect(parseHotkeyEvent(keyEvent({ key: "Shift", code: "ShiftLeft", shiftKey: true }))).toBeNull();
    expect(parseHotkeyEvent(keyEvent({ key: "Meta", code: "MetaLeft", metaKey: true }))).toBeNull();
  });

  it("supports function and digit keys", () => {
    expect(parseHotkeyEvent(keyEvent({ key: "F12", code: "F12", metaKey: true }))).toBe(
      "CommandOrControl+F12",
    );
    expect(parseHotkeyEvent(keyEvent({ key: "3", code: "Digit3", metaKey: true, shiftKey: true }))).toBe(
      "CommandOrControl+Shift+3",
    );
  });
});
