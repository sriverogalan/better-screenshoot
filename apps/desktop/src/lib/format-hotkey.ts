const MODIFIER_SYMBOLS: Record<string, string> = {
  CommandOrControl: "⌘",
  Command: "⌘",
  Control: "⌃",
  Shift: "⇧",
  Option: "⌥",
  Alt: "⌥",
};

export function formatHotkey(shortcut: string): string {
  return shortcut
    .split("+")
    .map((part) => MODIFIER_SYMBOLS[part] ?? part)
    .join("");
}
