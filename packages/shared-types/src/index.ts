export interface DisplayInfo {
  id: number;
  name: string;
  width: number;
  height: number;
  scale_factor: number;
  is_primary: boolean;
  x: number;
  y: number;
}

export interface Region {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface CaptureImage {
  width: number;
  height: number;
  png_bytes: number[];
}

export interface CaptureRecord {
  id: string;
  file_path: string;
  width: number;
  height: number;
  created_at: string;
  tags: string[];
}

export type CaptureMode = "screen" | "area";

export type SystemCaptureMode = "independent" | "replace_system";

export interface AppSettings {
  save_directory: string;
  auto_copy: boolean;
  auto_save: boolean;
  allow_external_control: boolean;
  system_capture_mode: SystemCaptureMode;
  hotkeys: HotkeyConfig;
  tier: LicenseTier;
  locale: AppLocale;
  onboarding_completed: boolean;
  appearance: AppAppearance;
}

export interface HotkeyConfig {
  capture_area: string;
  capture_screen: string;
  open_history: string;
}

export type LicenseTier = "community" | "pro" | "cloud" | "team";

export type AppAppearance = "auto" | "light" | "dark";

export type AppLocale = "en" | "es" | "fr" | "de" | "pt" | "it";

export type DeepLinkAction =
  | "capture-area"
  | "capture-screen"
  | "open-history"
  | "open-settings";

export const DEFAULT_HOTKEYS: HotkeyConfig = {
  capture_area: "CommandOrControl+Shift+X",
  capture_screen: "CommandOrControl+Shift+Option+S",
  open_history: "CommandOrControl+Shift+H",
};

/** Shortcuts that replace native macOS captures. */
export const SYSTEM_REPLACEMENT_HOTKEYS = {
  capture_screen: "Command+Shift+3",
  capture_area: "Command+Shift+4",
} as const satisfies Pick<HotkeyConfig, "capture_screen" | "capture_area">;

export const DEFAULT_SETTINGS: AppSettings = {
  save_directory: "",
  auto_copy: true,
  auto_save: true,
  allow_external_control: true,
  system_capture_mode: "independent",
  hotkeys: DEFAULT_HOTKEYS,
  tier: "community",
  locale: "en",
  onboarding_completed: false,
  appearance: "auto",
};
