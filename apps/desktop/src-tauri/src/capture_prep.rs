use tauri::{AppHandle, Manager, WebviewWindow};

use crate::window_layout;

const HIDE_POLL_INTERVAL_MS: u64 = 20;
const HIDE_TIMEOUT_MS: u64 = 400;
const HIDE_TIMEOUT_HUB_VISIBLE_MS: u64 = 700;

pub async fn hide_app_windows_before_capture(app: &AppHandle) {
    hide_windows(app, &["main", "editor", "overlay"]).await;
}

async fn hide_windows(app: &AppHandle, labels: &[&str]) {
    let mut requested_hide: Vec<String> = Vec::new();
    let mut hub_was_visible = false;

    for label in labels {
        if let Some(window) = app.get_webview_window(label) {
            if *label == "main" {
                if window.is_visible().unwrap_or(false) {
                    hub_was_visible = true;
                }
                window_layout::set_main_editor_mode(false);
                force_hide_hub_window(&window);
                requested_hide.push((*label).to_string());
            } else if window.is_visible().unwrap_or(false) {
                if *label == "editor" {
                    // Salir de la pantalla completa nativa antes de ocultar para no dejar Spaces vacíos.
                    #[cfg(target_os = "macos")]
                    let _ = window.set_simple_fullscreen(false);
                    let _ = window.set_fullscreen(false);
                }
                let _ = window.hide();
                requested_hide.push((*label).to_string());
            }
        }
    }

    if requested_hide.is_empty() {
        return;
    }

    let timeout_ms = if hub_was_visible {
        HIDE_TIMEOUT_HUB_VISIBLE_MS
    } else {
        HIDE_TIMEOUT_MS
    };
    wait_until_windows_hidden(app, &requested_hide, timeout_ms).await;
}

fn force_hide_hub_window(window: &WebviewWindow) {
    #[cfg(target_os = "macos")]
    let _ = window.set_simple_fullscreen(false);
    let _ = window.set_fullscreen(false);
    let _ = window.hide();
}

async fn wait_until_windows_hidden(app: &AppHandle, labels: &[String], timeout_ms: u64) {
    let deadline =
        tokio::time::Instant::now() + tokio::time::Duration::from_millis(timeout_ms);

    loop {
        let still_visible = labels.iter().any(|label| {
            app.get_webview_window(label)
                .map(|window| window.is_visible().unwrap_or(false))
                .unwrap_or(false)
        });

        if !still_visible {
            return;
        }

        if tokio::time::Instant::now() >= deadline {
            for label in labels {
                if let Some(window) = app.get_webview_window(label) {
                    if window.is_visible().unwrap_or(false) {
                        crate::app_trace!(
                            "capture_prep: window '{label}' still visible after {timeout_ms}ms"
                        );
                    }
                }
            }
            return;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(HIDE_POLL_INTERVAL_MS)).await;
    }
}

pub async fn hide_overlay_before_capture(app: &AppHandle) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        if overlay.is_visible().unwrap_or(false) {
            let _ = overlay.hide();
            wait_until_windows_hidden(app, &["overlay".to_string()], HIDE_TIMEOUT_MS).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HIDE_POLL_INTERVAL_MS, HIDE_TIMEOUT_MS};

    fn hub_obstructing(visible: bool, fullscreen: bool, maximized: bool) -> bool {
        visible || fullscreen || maximized
    }

    #[test]
    fn hide_poll_constants_are_reasonable() {
        assert!(HIDE_POLL_INTERVAL_MS > 0);
        assert!(HIDE_TIMEOUT_MS >= HIDE_POLL_INTERVAL_MS);
        assert!(HIDE_TIMEOUT_MS <= 1000);
    }

    #[test]
    fn hub_obstructing_when_visible_fullscreen_or_maximized() {
        assert!(hub_obstructing(true, false, false));
        assert!(hub_obstructing(false, true, false));
        assert!(hub_obstructing(false, false, true));
        assert!(!hub_obstructing(false, false, false));
    }
}
