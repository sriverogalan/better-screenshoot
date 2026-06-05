use tauri::{AppHandle, Manager};

const HIDE_DELAY_MS: u64 = 16;

pub async fn hide_app_windows_before_capture(app: &AppHandle) {
    hide_windows(app, &["main", "editor", "overlay"]).await;
}

async fn hide_windows(app: &AppHandle, labels: &[&str]) {
    let mut hid_any = false;
    for label in labels {
        if let Some(window) = app.get_webview_window(label) {
            if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
                hid_any = true;
            }
        }
    }
    if hid_any {
        tokio::time::sleep(tokio::time::Duration::from_millis(HIDE_DELAY_MS)).await;
    }
}

pub async fn hide_overlay_before_capture(app: &AppHandle) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        if overlay.is_visible().unwrap_or(false) {
            let _ = overlay.hide();
            tokio::time::sleep(tokio::time::Duration::from_millis(HIDE_DELAY_MS)).await;
        }
    }
}
