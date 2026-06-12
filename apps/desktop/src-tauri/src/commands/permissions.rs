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
            "macosPermissionRequired".into()
        } else {
            "macosPermissionRequired".into()
        };
        return (code, None);
    }

    #[cfg(not(target_os = "macos"))]
    {
        ("noDisplaysDetected".into(), None)
    }
}

#[tauri::command]
pub async fn get_capture_status(state: State<'_, AppState>) -> Result<CaptureStatus, String> {
    let displays = state.provider.list_displays().map_err(|_| app_error("captureFailed"))?;
    let granted = macos_screen_capture_granted();
    let (message_code, message_params) = permission_message_code(displays.len(), granted);

    Ok(CaptureStatus {
        displays_found: displays.len(),
        screen_capture_granted: granted && !displays.is_empty(),
        message_code,
        message_params,
        dev_binary_path: dev_binary_hint(),
    })
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
