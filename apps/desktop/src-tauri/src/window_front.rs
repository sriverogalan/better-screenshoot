use tauri::{Manager, WebviewWindow};

use crate::window_activation::activate_app_for_window;

/// Muestra la ventana por encima del resto de apps (tray/background en macOS).
pub async fn bring_window_to_front(window: &WebviewWindow) {
    activate_app_for_window(&window.app_handle());

    let _ = window.unminimize();
    if let Err(_error) = window.show() {
        crate::app_trace!("window.show() failed for {}: {error}", window.label());
    }

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

/// Presentación del editor: activa la app y trae la ventana al frente.
#[allow(dead_code)]
pub async fn bring_editor_to_front(window: &WebviewWindow) {
    bring_editor_to_front_inner(window, true).await;
}

/// Enfoca el editor sin reactivar la app (evita que macOS restaure el hub en Ajustes).
pub async fn bring_editor_to_front_quiet(window: &WebviewWindow) {
    bring_editor_to_front_inner(window, false).await;
}

async fn bring_editor_to_front_inner(window: &WebviewWindow, activate_app: bool) {
    if activate_app {
        activate_app_for_window(&window.app_handle());
    }

    let _ = window.unminimize();
    if let Err(_error) = window.show() {
        crate::app_trace!("editor.show() failed: {error}");
    }

    #[cfg(target_os = "macos")]
    {
        let _ = window.set_always_on_top(true);
    }

    let _ = window.set_focus();

    #[cfg(target_os = "macos")]
    {
        tokio::time::sleep(tokio::time::Duration::from_millis(120)).await;
        let _ = window.set_always_on_top(false);
        let _ = window.set_focus();
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = window.set_focus();
    }
}
