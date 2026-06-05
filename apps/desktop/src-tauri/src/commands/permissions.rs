use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct CaptureStatus {
    pub displays_found: usize,
    pub screen_capture_granted: bool,
    pub message: String,
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

fn permission_message(displays_found: usize, granted: bool) -> String {
    if displays_found > 0 {
        return format!("{displays_found} display(s) detected.");
    }

    #[cfg(target_os = "macos")]
    {
        let mut message = String::from(
            "macOS does not allow screen capture. Go to System Settings → Privacy & Security → Screen Recording and enable Better Screenshoot.",
        );
        if !granted {
            message.push_str(" If you use `pnpm dev`, also authorize the binary at target/debug/better-screenshoot.");
        }
        return message;
    }

    #[cfg(not(target_os = "macos"))]
    {
        "No displays detected.".into()
    }
}

#[tauri::command]
pub async fn get_capture_status(state: State<'_, AppState>) -> Result<CaptureStatus, String> {
    let displays = state.provider.list_displays().map_err(|e| e.to_string())?;
    let granted = macos_screen_capture_granted();

    Ok(CaptureStatus {
        displays_found: displays.len(),
        screen_capture_granted: granted && !displays.is_empty(),
        message: permission_message(displays.len(), granted),
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
            .map_err(|e| format!("Could not open system settings: {e}"))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("System capture shortcuts can only be configured on macOS.".into())
    }
}
