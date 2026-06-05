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

export interface WindowInfo {
  id: number;
  title: string;
  app_name: string;
  width: number;
  height: number;
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

export type CaptureMode = "screen" | "window" | "area";

export interface AppSettings {
  save_directory: string;
  auto_copy: boolean;
  auto_save: boolean;
  allow_external_control: boolean;
  replace_system_screenshots: boolean;
  hotkeys: HotkeyConfig;
  tier: LicenseTier;
}

export interface HotkeyConfig {
  capture_area: string;
  capture_screen: string;
  capture_window: string;
  open_history: string;
}

export type LicenseTier = "community" | "pro" | "cloud" | "team";

export type DeepLinkAction =
  | "capture-area"
  | "capture-screen"
  | "capture-window"
  | "open-history"
  | "open-settings";

export const DEFAULT_HOTKEYS: HotkeyConfig = {
  capture_area: "CommandOrControl+Shift+X",
  capture_screen: "CommandOrControl+Shift+Option+S",
  capture_window: "CommandOrControl+Shift+Option+W",
  open_history: "CommandOrControl+Shift+H",
};

export const DEFAULT_SETTINGS: AppSettings = {
  save_directory: "",
  auto_copy: true,
  auto_save: true,
  allow_external_control: true,
  replace_system_screenshots: false,
  hotkeys: DEFAULT_HOTKEYS,
  tier: "community",
};
