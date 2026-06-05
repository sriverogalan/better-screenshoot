use tauri::WebviewWindow;

/// Muestra la ventana por encima del resto de apps (tray/background en macOS).
pub async fn bring_window_to_front(window: &WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.show();

    #[cfg(target_os = "macos")]
    {
        let _ = window.set_always_on_top(true);
    }

    let _ = window.set_focus();

    #[cfg(target_os = "macos")]
    {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let _ = window.set_always_on_top(false);
    }
}
