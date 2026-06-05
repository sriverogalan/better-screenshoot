import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  CaptureRecord,
  DisplayInfo,
  Region,
  WindowInfo,
} from "@better-screenshoot/shared-types";

export interface SavedCapture {
  id: string;
  file_path: string;
  width: number;
  height: number;
  created_at: string;
  data_url: string;
}

export async function listDisplays(): Promise<DisplayInfo[]> {
  return invoke("list_displays");
}

export async function listWindows(): Promise<WindowInfo[]> {
  return invoke("list_windows");
}

export interface CaptureStatus {
  displays_found: number;
  screen_capture_granted: boolean;
  message: string;
  dev_binary_path: string | null;
}

export async function getCaptureStatus(): Promise<CaptureStatus> {
  return invoke("get_capture_status");
}

export async function requestScreenCapturePermission(): Promise<boolean> {
  return invoke("request_screen_capture_permission");
}

export async function captureScreen(displayId?: number): Promise<SavedCapture> {
  return invoke("capture_screen", { displayId });
}

export async function captureWindow(windowId: number): Promise<SavedCapture> {
  return invoke("capture_window", { windowId });
}

export async function captureRegion(
  displayId: number,
  region: Region,
): Promise<SavedCapture> {
  return invoke("capture_region", { displayId, region });
}

export interface OverlayPreview {
  preview_path: string;
  width: number;
  height: number;
  source_width: number;
  source_height: number;
  display_id: number;
  scale_factor: number;
}

export async function getOverlayPreview(displayId?: number): Promise<OverlayPreview> {
  return invoke("get_overlay_preview", { displayId });
}

export async function completeAreaCapture(
  displayId: number,
  region: Region,
): Promise<SavedCapture> {
  return invoke("complete_area_capture", { displayId, region });
}

export async function captureAreaInteractive(): Promise<SavedCapture> {
  return invoke("capture_area_interactive");
}

export async function takePendingCapture(): Promise<SavedCapture | null> {
  return invoke("take_pending_capture");
}

export async function clearPendingCapture(): Promise<void> {
  return invoke("clear_pending_capture");
}

export async function openCaptureInEditor(captureId: string): Promise<void> {
  return invoke("open_capture_in_editor", { captureId });
}

export function captureImageSrc(capture: SavedCapture): string {
  if (capture.file_path) {
    return convertFileSrc(capture.file_path);
  }
  return capture.data_url;
}

export async function captureViaPortal(): Promise<SavedCapture> {
  return invoke("capture_via_portal");
}

export async function getHistory(limit = 100): Promise<CaptureRecord[]> {
  const rows = await invoke<
    Array<{
      id: string;
      file_path: string;
      width: number;
      height: number;
      created_at: string;
      tags: string;
    }>
  >("get_history", { limit });

  return rows.map((row) => ({
    id: row.id,
    file_path: row.file_path,
    width: row.width,
    height: row.height,
    created_at: row.created_at,
    tags: JSON.parse(row.tags || "[]") as string[],
  }));
}

export async function deleteHistoryItem(id: string): Promise<void> {
  return invoke("delete_history_item", { id });
}

export async function discardCapture(
  captureId: string,
  filePath: string,
): Promise<void> {
  return invoke("discard_capture", { captureId, filePath });
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function updateSettings(settings: AppSettings): Promise<AppSettings> {
  return invoke("update_settings", { settings });
}

export async function copyImageToClipboard(pngBase64: string): Promise<void> {
  return invoke("copy_image_to_clipboard", { pngBase64 });
}

export async function saveImageToDisk(pngBase64: string): Promise<SavedCapture> {
  return invoke("save_image_to_disk", { pngBase64 });
}

export async function saveImageWithDialog(
  pngBase64: string,
): Promise<SavedCapture | null> {
  return invoke("save_image_with_dialog", { pngBase64 });
}

export interface LicenseValidationResult {
  valid: boolean;
  tier: string;
  expires_at: string | null;
  message: string;
}

export async function validateLicenseKey(key: string): Promise<LicenseValidationResult> {
  return invoke("validate_license_key", { key });
}

export async function uploadForShare(filePath: string): Promise<{ url: string; expires_at: string }> {
  return invoke("upload_for_share", { filePath });
}
