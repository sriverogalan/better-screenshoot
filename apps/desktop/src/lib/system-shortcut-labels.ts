const SYSTEM_SHORTCUT_LABEL_KEYS: Record<number, string> = {
  28: "systemShortcuts.captureScreen",
  29: "systemShortcuts.copyScreen",
  30: "systemShortcuts.captureRegion",
  31: "systemShortcuts.copyRegion",
};

export function systemShortcutLabelKey(id: number): string | null {
  return SYSTEM_SHORTCUT_LABEL_KEYS[id] ?? null;
}
