mod app_log;
mod capture_prep;
mod capture_session;
mod commands;
mod deep_link;
mod errors;
mod shortcuts;
mod state;
mod system_capture;
mod system_shortcuts;
mod tray;
mod window_activation;
mod window_front;
mod window_layout;

use tauri::{Emitter, Manager};
use tauri_plugin_single_instance::init as single_instance;

use state::{load_settings, save_settings, AppState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations(
                    "sqlite:better-screenshoot.db",
                    commands::history::migrations(),
                )
                .build(),
        )
        .plugin(single_instance(|app, args, _cwd| {
            if let Some(url) = args.iter().find(|a| a.starts_with("betterscreenshoot://")) {
                deep_link::handle_deep_link(app, vec![url.clone()]);
            } else if !capture_session::is_active_fresh() {
                if let Some(main) = app.get_webview_window("main") {
                    let hub_epoch = capture_session::current_hub_show_epoch();
                    let main_handle = main.clone();
                    tauri::async_runtime::spawn(async move {
                        if !capture_session::should_show_hub(hub_epoch) {
                            return;
                        }
                        let _ = window_layout::prepare_main_hub_window(&main_handle);
                        if !capture_session::should_show_hub(hub_epoch) {
                            return;
                        }
                        crate::window_front::bring_window_to_front(&main_handle).await;
                    });
                }
            }
        }))
        .setup(|app| {
            let handle = app.handle().clone();
            let state = app.state::<AppState>();
            load_settings(&handle, state.inner())?;
            tray::setup_tray(&handle)?;

            if let Some(message) = commands::reconcile_system_capture(&handle, state.inner())? {
                let _ = handle.emit("system-capture-drift", message);
            }

            shortcuts::register_hotkeys(&handle)?;

            #[cfg(any(target_os = "linux", target_os = "windows"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let _ = app.deep_link().register("betterscreenshoot");
            }

            if let Some(main) = app.get_webview_window("main") {
                let _ = window_layout::prepare_main_hub_window(&main);
            }

            // Warm up the editor webview (Tauri may defer loading if it was never shown).
            if let Some(editor) = app.get_webview_window("editor") {
                window_activation::activate_app_for_window(&handle);
                let _ = editor.show();
                let _ = editor.hide();
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                    return;
                }

                if matches!(
                    event,
                    tauri::WindowEvent::Focused(true) | tauri::WindowEvent::Resized(_)
                ) {
                    window_layout::watch_main_hub_window(window);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_displays,
            commands::list_windows,
            commands::capture_screen,
            commands::capture_window,
            commands::capture_region,
            commands::get_overlay_preview,
            commands::complete_area_capture,
            commands::capture_area_interactive,
            commands::capture_via_portal,
            commands::peek_pending_capture,
            commands::take_pending_capture,
            commands::clear_pending_capture,
            commands::open_pending_capture_in_editor,
            commands::open_capture_in_editor,
            commands::read_capture_data_url,
            commands::discard_capture,
            commands::copy_image_to_clipboard,
            commands::save_image_to_disk,
            commands::save_image_with_dialog,
            commands::get_history,
            commands::delete_history_item,
            commands::get_settings,
            commands::update_settings,
            commands::reload_settings,
            commands::validate_license_key,
            commands::upload_for_share,
            commands::get_capture_status,
            commands::request_screen_capture_permission,
            commands::open_system_screenshot_shortcuts_settings,
            commands::get_system_capture_status,
            commands::set_system_capture_mode,
            window_layout::reset_editor_window_layout,
            window_layout::reset_main_window_layout,
            window_layout::exit_main_editor_mode,
            shortcuts::handle_capture_action,
            tray::rebuild_tray_menu,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app, event| {
            match event {
                tauri::RunEvent::ExitRequested { api, .. } => {
                    api.prevent_exit();
                }
                tauri::RunEvent::WindowEvent { label, event, .. } => {
                    if label == "main" && matches!(event, tauri::WindowEvent::Destroyed) {
                        let state = app.state::<AppState>();
                        let _ = save_settings(app, state.inner());
                    }
                }
                _ => {}
            }
        });
}
