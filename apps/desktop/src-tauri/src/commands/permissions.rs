use serde::Serialize;
use tauri::State;

use crate::errors::app_error;
use crate::state::AppState;

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

fn permission_message_code(displays_found: usize, granted: bool) -> (String, Option<serde_json::Value>) {
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

/// Computes a [`CaptureStatus`] from raw display list and TCC grant state.
/// Extracted so unit tests can exercise the full status-building logic without
/// needing a live Tauri [`State`].
fn compute_capture_status(displays: Vec<capture_core::types::DisplayInfo>, granted: bool) -> CaptureStatus {
    // If xcap listed at least one display, ScreenCaptureKit already validated the permission —
    // CGPreflightScreenCaptureAccess is unreliable on macOS Sequoia (returns false despite TCC grant).
    let screen_capture_granted = !displays.is_empty();
    let (message_code, message_params) = permission_message_code(displays.len(), granted);
    CaptureStatus {
        displays_found: displays.len(),
        screen_capture_granted,
        message_code,
        message_params,
        dev_binary_path: dev_binary_hint(),
    }
}

#[tauri::command]
pub async fn get_capture_status(state: State<'_, AppState>) -> Result<CaptureStatus, String> {
    let displays = state.provider.list_displays().unwrap_or_default();
    let granted = macos_screen_capture_granted();
    Ok(compute_capture_status(displays, granted))
}

#[tauri::command]
pub fn request_screen_capture_permission() -> bool {
    macos_request_screen_capture()
}

#[cfg(target_os = "macos")]
const SYSTEM_SCREENSHOT_SHORTCUTS_URL: &str =
    "x-apple.systempreferences:com.apple.Keyboard-Settings.extension?Screenshots";

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
        let status = compute_capture_status(vec![], false);
        assert_eq!(status.message_code, "macosPermissionRequired");
        assert!(!status.screen_capture_granted);
        assert_eq!(status.displays_found, 0);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn capture_status_no_displays_granted_returns_granted_no_displays() {
        // TCC says granted but xcap returned empty (e.g. needs restart on Sequoia)
        let status = compute_capture_status(vec![], true);
        assert_eq!(status.message_code, "macosPermissionGrantedNoDisplays");
        assert!(!status.screen_capture_granted);
        assert_eq!(status.displays_found, 0);
    }

    #[test]
    fn capture_status_with_displays_is_granted_regardless_of_preflight() {
        use capture_core::types::DisplayInfo;
        let display = DisplayInfo { id: 1, name: "Test".into(), width: 1920, height: 1080, scale_factor: 1.0, is_primary: true, x: 0, y: 0 };
        // granted=false simulates Sequoia CGPreflightScreenCaptureAccess quirk
        let status = compute_capture_status(vec![display], false);
        assert_eq!(status.message_code, "displaysDetected");
        assert!(status.screen_capture_granted); // xcap worked → granted
        assert_eq!(status.displays_found, 1);
    }
}
