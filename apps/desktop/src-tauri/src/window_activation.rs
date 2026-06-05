use tauri::AppHandle;

/// On macOS, a tray app must activate before a new window becomes visible.
pub fn activate_app_for_window(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        use tauri::ActivationPolicy;

        if let Err(_error) = app.set_activation_policy(ActivationPolicy::Regular) {
            crate::app_trace!("set_activation_policy failed: {error}");
        }
        if let Err(_error) = app.set_dock_visibility(true) {
            crate::app_trace!("set_dock_visibility failed: {error}");
        }
    }
}

/// Returns to accessory mode when no windows remain visible (tray).
#[allow(dead_code)]
pub fn deactivate_app_to_accessory(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        use tauri::{ActivationPolicy, Manager};

        let any_visible = ["main", "editor", "overlay"].iter().any(|label| {
            app.get_webview_window(label)
                .map(|window| window.is_visible().unwrap_or(false))
                .unwrap_or(false)
        });

        if any_visible {
            return;
        }

        if let Err(_error) = app.set_activation_policy(ActivationPolicy::Accessory) {
            crate::app_trace!("set_activation_policy(accessory) failed: {error}");
        }
    }
}
