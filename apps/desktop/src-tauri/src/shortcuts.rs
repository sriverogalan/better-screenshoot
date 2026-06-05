use tauri::{AppHandle, Emitter, Manager};
#[cfg(target_os = "windows")]
use tauri::WebviewWindow;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::capture_session;
use crate::state::AppState;
use crate::window_front::bring_window_to_front;
use crate::window_layout::prepare_main_hub_window;

pub fn register_hotkeys(app: &AppHandle) -> Result<(), String> {
    let settings = {
        let state = app.state::<AppState>();
        let guard = state.settings.lock().map_err(|e| e.to_string())?;
        guard.hotkeys.clone()
    };

    let gs = app.global_shortcut();
    let _ = gs.unregister_all();

    // Un fallo al registrar un atajo no debe impedir registrar los demás.
    register_one(app, &settings.capture_area, "capture-area");
    register_one(app, &settings.capture_screen, "capture-screen");
    register_one(app, &settings.capture_window, "capture-window");
    register_one(app, &settings.open_history, "open-history");

    Ok(())
}

fn register_one(app: &AppHandle, shortcut: &str, action: &str) {
    let action_owned = action.to_string();
    let app_handle = app.clone();

    match app
        .global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                handle_hotkey_action(&app_handle, &action_owned);
            }
        }) {
        Ok(()) => crate::app_trace!("register_hotkey: OK '{shortcut}' -> {action}"),
        Err(_error) => crate::app_trace!("register_hotkey: ERROR '{shortcut}' -> {action}"),
    }
}

pub fn handle_hotkey_action(app: &AppHandle, action: &str) {
    match action {
        "capture-area" => start_area_capture(app),
        "capture-screen" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let gen = capture_session::begin(&app);
                if gen == 0 {
                    crate::app_trace!("capture-screen: captura ya en curso, ignorando");
                    return;
                }
                crate::capture_prep::hide_app_windows_before_capture(&app).await;
                if let Err(error) =
                    crate::commands::capture::capture_screen_internal(app.clone(), None).await
                {
                    capture_session::end_generation(&app, gen);
                    let _ = app.emit("capture-error", error);
                }
            });
        }
        "capture-window" => {
            let _ = app.emit("open-window-picker", ());
            show_main_window(app, "/capture-window");
        }
        "open-history" => show_main_window(app, "/history"),
        _ => {}
    }
}

#[cfg(target_os = "windows")]
fn prepare_overlay_window(overlay: &WebviewWindow) {
    let _ = overlay.set_decorations(false);
    let _ = overlay.set_always_on_top(true);
    let _ = overlay.set_skip_taskbar(true);
    #[cfg(target_os = "macos")]
    {
        use tauri::window::Color;
        let _ = overlay.set_background_color(Some(Color(0, 0, 0, 0)));
    }
}

pub fn start_area_capture(app: &AppHandle) {
    #[cfg(target_os = "windows")]
    {
        show_overlay(app);
        return;
    }

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let gen = capture_session::begin(&app);
        if gen == 0 {
            crate::app_trace!("start_area_capture: captura ya en curso, ignorando");
            return;
        }
        crate::capture_prep::hide_app_windows_before_capture(&app).await;
        match crate::commands::capture::capture_area_interactive_internal(app.clone()).await {
            Ok(_) => {}
            Err(error) if error != "Captura cancelada" => {
                capture_session::end_generation(&app, gen);
                let _ = app.emit("capture-error", error);
            }
            Err(_) => {
                capture_session::end_generation(&app, gen);
            }
        }
    });
}

#[cfg(target_os = "windows")]
pub fn show_overlay(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let overlay = match app.get_webview_window("overlay") {
            Some(window) => window,
            None => return,
        };

        prepare_overlay_window(&overlay);
        let _ = overlay.emit("overlay-loading", ());
        let _ = overlay.set_fullscreen(true);
        let _ = overlay.show();
        let _ = overlay.set_focus();
        tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;

        crate::capture_prep::hide_app_windows_before_capture(&app).await;

        match crate::commands::capture::capture_overlay_preview_internal(&app, None).await {
            Ok(preview) => {
                let _ = overlay.emit("overlay-preview", &preview);
            }
            Err(error) => {
                let _ = overlay.emit("overlay-error", &error);
                let _ = app.emit("capture-error", error);
            }
        }

        let _ = overlay.set_fullscreen(true);
        let _ = overlay.show();
        let _ = overlay.set_focus();
    });
}

#[tauri::command]
pub fn handle_capture_action(app: AppHandle, action: String) {
    handle_hotkey_action(&app, &action);
}

pub fn show_main_window(app: &AppHandle, route: &str) {
    if capture_session::is_active_fresh() || crate::window_layout::is_main_editor_mode() {
        return;
    }

    let Some(main) = app.get_webview_window("main") else {
        return;
    };

    let hub_epoch = capture_session::current_hub_show_epoch();
    let route = route.to_string();

    if let Some(editor) = app.get_webview_window("editor") {
        if editor.is_visible().unwrap_or(false) {
            return;
        }
        let _ = editor.hide();
    }
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.hide();
    }

    let main_handle = main.clone();
    tauri::async_runtime::spawn(async move {
        if !capture_session::should_show_hub(hub_epoch) {
            return;
        }

        let _ = prepare_main_hub_window(&main_handle);
        if !capture_session::should_show_hub(hub_epoch) {
            return;
        }

        bring_window_to_front(&main_handle).await;

        if !capture_session::should_show_hub(hub_epoch) {
            return;
        }

        let _ = main_handle.emit("navigate", route);
    });
}
