use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::errors::app_error;
use crate::state::AppState;

#[cfg(target_os = "macos")]
const APP_BUNDLE_IDENTIFIER: &str = "com.betterscreenshoot.app";

#[derive(Debug, Serialize)]
pub struct CaptureStatus {
    pub displays_found: usize,
    pub screen_capture_granted: bool,
    #[serde(rename = "messageCode")]
    pub message_code: String,
    #[serde(rename = "messageParams", skip_serializing_if = "Option::is_none")]
    pub message_params: Option<serde_json::Value>,
    pub dev_binary_path: Option<String>,
}

#[cfg(target_os = "macos")]
mod macos_screen_capture {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        pub fn CGPreflightScreenCaptureAccess() -> bool;
        pub fn CGRequestScreenCaptureAccess() -> bool;
    }
}

#[cfg(target_os = "macos")]
fn macos_screen_capture_granted() -> bool {
    unsafe { macos_screen_capture::CGPreflightScreenCaptureAccess() }
}

#[cfg(not(target_os = "macos"))]
fn macos_screen_capture_granted() -> bool {
    true
}

#[cfg(target_os = "macos")]
fn macos_request_screen_capture() -> bool {
    unsafe { macos_screen_capture::CGRequestScreenCaptureAccess() }
}

#[cfg(not(target_os = "macos"))]
fn macos_request_screen_capture() -> bool {
    true
}

fn dev_binary_hint() -> Option<String> {
    std::env::current_exe()
        .ok()
        .map(|path| path.display().to_string())
}

fn permission_message_code(
    displays_found: usize,
    granted: bool,
) -> (String, Option<serde_json::Value>) {
    if displays_found > 0 {
        return (
            "displaysDetected".into(),
            Some(serde_json::json!({ "count": displays_found })),
        );
    }

    #[cfg(target_os = "macos")]
    {
        let code = if granted {
            "macosPermissionGrantedNoDisplays"
        } else {
            "macosPermissionRequired"
        };
        return (code.into(), None);
    }

    #[cfg(not(target_os = "macos"))]
    {
        ("noDisplaysDetected".into(), None)
    }
}

/// Computes a [`CaptureStatus`] from raw display list, TCC grant state, and (when the two
/// disagree) a real-capture probe result.
///
/// `xcap`'s display listing (`CGGetActiveDisplayList`) never requires Screen Recording
/// permission, so `displays` being non-empty is NOT proof that capture is authorized — it is
/// true on every Mac regardless of TCC state. Treating it as such (as a previous version of
/// this function did) let the app silently attempt captures that macOS answers with a solid
/// black image, with no error and no warning to the user.
///
/// `granted` (`CGPreflightScreenCaptureAccess`) is the real signal, but has a known false-negative
/// quirk on some macOS versions (reports `false` right after the user grants access, until the
/// app restarts). When it disagrees with the display list, `verified_by_probe` — the result of
/// an actual test capture checked for non-uniform pixel content — breaks the tie.
///
/// Extracted so unit tests can exercise the full status-building logic without
/// needing a live Tauri [`State`].
fn compute_capture_status(
    displays: Vec<capture_core::types::DisplayInfo>,
    granted: bool,
    verified_by_probe: bool,
) -> CaptureStatus {
    let screen_capture_granted = !displays.is_empty() && (granted || verified_by_probe);
    let (message_code, message_params) = permission_message_code(displays.len(), granted);
    CaptureStatus {
        displays_found: displays.len(),
        screen_capture_granted,
        message_code,
        message_params,
        dev_binary_path: dev_binary_hint(),
    }
}

/// Performs a real capture of the primary display and checks whether the result has any
/// pixel variation. A macOS process without Screen Recording access gets back a uniformly
/// black image from `CGWindowListCreateImage` (no error) — real content, even on a plain
/// desktop, always has at least the menu bar/cursor/dock rendered, which is never a flat color.
#[cfg(target_os = "macos")]
fn probe_real_capture(app: &AppHandle) -> bool {
    let state = app.state::<AppState>();
    let Ok(displays) = state.provider.list_displays() else {
        return false;
    };
    let Some(display) = displays.iter().find(|d| d.is_primary).or(displays.first()) else {
        return false;
    };
    let Ok(rgba) = state.provider.capture_display_rgba(display.id) else {
        return false;
    };
    let pixels = rgba.as_raw();
    match pixels.chunks_exact(4).next() {
        Some(first) => pixels.chunks_exact(4).any(|pixel| pixel != first),
        None => false,
    }
}

#[cfg(not(target_os = "macos"))]
fn probe_real_capture(_app: &AppHandle) -> bool {
    false
}

#[tauri::command]
pub async fn get_capture_status(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CaptureStatus, String> {
    let displays = state.provider.list_displays().unwrap_or_default();
    let granted = macos_screen_capture_granted();

    // Only pay for a real test capture when the cheap signals disagree.
    let verified_by_probe = if !displays.is_empty() && !granted {
        let app_probe = app.clone();
        tauri::async_runtime::spawn_blocking(move || probe_real_capture(&app_probe))
            .await
            .unwrap_or(false)
    } else {
        false
    };

    Ok(compute_capture_status(displays, granted, verified_by_probe))
}

#[tauri::command]
pub fn request_screen_capture_permission() -> bool {
    macos_request_screen_capture()
}

#[tauri::command]
pub fn reset_screen_capture_permission() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("tccutil")
            .args(["reset", "ScreenCapture", APP_BUNDLE_IDENTIFIER])
            .status()
            .map_err(|_| app_error("repairScreenRecordingPermissionFailed"))?;

        if status.success() {
            Ok(())
        } else {
            Err(app_error("repairScreenRecordingPermissionFailed"))
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
}

#[cfg(target_os = "macos")]
const SYSTEM_SCREENSHOT_SHORTCUTS_URL: &str =
    "x-apple.systempreferences:com.apple.Keyboard-Settings.extension?Screenshots";

#[cfg(target_os = "macos")]
const SCREEN_RECORDING_PRIVACY_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture";

#[tauri::command]
pub fn open_screen_recording_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(SCREEN_RECORDING_PRIVACY_URL)
            .spawn()
            .map_err(|_| app_error("openSystemSettingsFailed"))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(app_error("macosOnly"))
    }
}

#[tauri::command]
pub fn open_system_screenshot_shortcuts_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(SYSTEM_SCREENSHOT_SHORTCUTS_URL)
            .spawn()
            .map_err(|_| app_error("openSystemSettingsFailed"))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(app_error("macosShortcutsOnly"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- 8.1: permission_message_code — all three branches ---

    #[test]
    fn permission_code_with_displays_returns_displays_detected() {
        let (code, params) = permission_message_code(3, true);
        assert_eq!(code, "displaysDetected");
        assert!(params.is_some());
        assert_eq!(params.unwrap()["count"], 3);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn permission_code_granted_no_displays_returns_granted_no_displays_code() {
        let (code, params) = permission_message_code(0, true);
        assert_eq!(code, "macosPermissionGrantedNoDisplays");
        assert!(params.is_none());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn permission_code_not_granted_returns_permission_required() {
        let (code, params) = permission_message_code(0, false);
        assert_eq!(code, "macosPermissionRequired");
        assert!(params.is_none());
    }

    // --- 8.2: compute_capture_status — simulates list_displays() returning Err via empty vec ---

    #[cfg(target_os = "macos")]
    #[test]
    fn capture_status_no_displays_not_granted_returns_permission_required() {
        let status = compute_capture_status(vec![], false, false);
        assert_eq!(status.message_code, "macosPermissionRequired");
        assert!(!status.screen_capture_granted);
        assert_eq!(status.displays_found, 0);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn capture_status_no_displays_granted_returns_granted_no_displays() {
        // TCC says granted but xcap returned empty (e.g. needs restart on Sequoia)
        let status = compute_capture_status(vec![], true, false);
        assert_eq!(status.message_code, "macosPermissionGrantedNoDisplays");
        assert!(!status.screen_capture_granted);
        assert_eq!(status.displays_found, 0);
    }

    fn test_display() -> capture_core::types::DisplayInfo {
        capture_core::types::DisplayInfo {
            id: 1,
            name: "Test".into(),
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
            is_primary: true,
            x: 0,
            y: 0,
        }
    }

    #[test]
    fn capture_status_with_displays_and_granted_preflight_is_granted() {
        let status = compute_capture_status(vec![test_display()], true, false);
        assert_eq!(status.message_code, "displaysDetected");
        assert!(status.screen_capture_granted);
        assert_eq!(status.displays_found, 1);
    }

    #[test]
    fn capture_status_with_displays_but_no_preflight_and_no_probe_is_not_granted() {
        // Listing displays never requires Screen Recording permission, so it alone must never
        // be treated as proof of a grant — otherwise capture silently returns a black image.
        let status = compute_capture_status(vec![test_display()], false, false);
        assert!(!status.screen_capture_granted);
    }

    #[test]
    fn capture_status_with_displays_no_preflight_but_probe_verified_is_granted() {
        // Preflight false-negative (e.g. right after granting, before app restart) is
        // resolved by an actual real-capture probe finding non-black content.
        let status = compute_capture_status(vec![test_display()], false, true);
        assert!(status.screen_capture_granted);
    }

    #[test]
    fn capture_status_no_displays_ignores_probe() {
        let status = compute_capture_status(vec![], false, true);
        assert!(!status.screen_capture_granted);
        assert_eq!(status.displays_found, 0);
    }
}
