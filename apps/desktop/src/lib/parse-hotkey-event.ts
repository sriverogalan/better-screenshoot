const MODIFIER_KEYS = new Set([
  "Meta",
  "Control",
  "Alt",
  "Shift",
  "OS",
  "Hyper",
  "Super",
]);

const CODE_TO_KEY: Record<string, string> = {
  Space: "Space",
  Minus: "-",
  Equal: "=",
  BracketLeft: "[",
  BracketRight: "]",
  Backslash: "\\",
  Semicolon: ";",
  Quote: "'",
  Comma: ",",
  Period: ".",
  Slash: "/",
  Backquote: "`",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  Escape: "Esc",
  Enter: "Enter",
  Tab: "Tab",
  Backspace: "Backspace",
  Delete: "Delete",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
};

/**
 * Converts a KeyboardEvent into a tauri-plugin-global-shortcut accelerator string.
 * Returns null for incomplete chords (modifiers only) or Escape (cancel).
 */
export function parseHotkeyEvent(event: KeyboardEvent): string | null {
  if (event.key === "Escape") return null;
  if (MODIFIER_KEYS.has(event.key)) return null;

  const parts: string[] = [];

  if (event.metaKey || event.ctrlKey) {
    parts.push("CommandOrControl");
  }
  if (event.shiftKey) {
    parts.push("Shift");
  }
  if (event.altKey) {
    parts.push("Option");
  }

  const key = keyFromEvent(event);
  if (!key) return null;

  parts.push(key);
  return parts.join("+");
}

function keyFromEvent(event: KeyboardEvent): string | null {
  if (/^F\d{1,2}$/.test(event.key)) {
    return event.key;
  }

  if (event.code.startsWith("Digit")) {
    return event.code.slice(5);
  }

  if (event.code.startsWith("Key")) {
    return event.code.slice(3);
  }

  if (event.code.startsWith("Numpad")) {
    const suffix = event.code.slice(6);
    if (/^\d$/.test(suffix)) return `Numpad${suffix}`;
    return CODE_TO_KEY[event.code] ?? suffix;
  }

  return CODE_TO_KEY[event.code] ?? (event.key.length === 1 ? event.key.toUpperCase() : event.key);
}
