mod capture_prep;
mod commands;
mod deep_link;
mod shortcuts;
mod state;
mod system_capture;
mod tray;
mod window_front;

use tauri::Manager;
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
            } else if let Some(main) = app.get_webview_window("main") {
                let main_handle = main.clone();
                tauri::async_runtime::spawn(async move {
                    crate::window_front::bring_window_to_front(&main_handle).await;
                });
            }
        }))
        .setup(|app| {
            let handle = app.handle().clone();
            let state = app.state::<AppState>();
            load_settings(&handle, state.inner())?;
            tray::setup_tray(&handle)?;
            shortcuts::register_hotkeys(&handle)?;

            #[cfg(any(target_os = "linux", target_os = "windows"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let _ = app.deep_link().register("betterscreenshoot");
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
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
            commands::take_pending_capture,
            commands::clear_pending_capture,
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
            shortcuts::handle_capture_action,
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
