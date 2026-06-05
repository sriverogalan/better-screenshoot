use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle,
};

use crate::shortcuts::{handle_hotkey_action, show_main_window};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let capture_area = MenuItem::with_id(app, "capture-area", "Capture Area", true, None::<&str>)?;
    let capture_screen =
        MenuItem::with_id(app, "capture-screen", "Capture Screen", true, None::<&str>)?;
    let capture_window =
        MenuItem::with_id(app, "capture-window", "Capture Window", true, None::<&str>)?;
    let history = MenuItem::with_id(app, "history", "History", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &capture_area,
            &capture_screen,
            &capture_window,
            &history,
            &settings,
            &quit,
        ],
    )?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().cloned().expect("tray icon"))
        .menu(&menu)
        .tooltip("Better Screenshoot")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "capture-area" => handle_hotkey_action(app, "capture-area"),
            "capture-screen" => handle_hotkey_action(app, "capture-screen"),
            "capture-window" => handle_hotkey_action(app, "capture-window"),
            "history" => show_main_window(app, "/history"),
            "settings" => show_main_window(app, "/settings"),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                show_main_window(&app, "/history");
            }
        })
        .build(app)?;

    Ok(())
}
