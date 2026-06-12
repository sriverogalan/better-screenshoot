use serde::Deserialize;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::shortcuts::{handle_hotkey_action, show_main_window};
use crate::state::{AppState, AppSettings};

#[derive(Debug, Deserialize)]
struct TrayLocaleFile {
    tray: TrayLabels,
}

#[derive(Debug, Deserialize)]
struct TrayLabels {
    tooltip: String,
    #[serde(rename = "captureArea")]
    capture_area: String,
    #[serde(rename = "captureScreen")]
    capture_screen: String,
    #[serde(rename = "captureWindow")]
    capture_window: String,
    history: String,
    settings: String,
    quit: String,
}

fn load_tray_labels(locale: &str) -> TrayLabels {
    let normalized = match locale {
        "en" | "es" | "fr" | "de" | "pt" | "it" => locale,
        _ => "en",
    };

    let json = match normalized {
        "es" => include_str!("../locales/es.json"),
        "fr" => include_str!("../locales/fr.json"),
        "de" => include_str!("../locales/de.json"),
        "pt" => include_str!("../locales/pt.json"),
        "it" => include_str!("../locales/it.json"),
        _ => include_str!("../locales/en.json"),
    };

    serde_json::from_str::<TrayLocaleFile>(json)
        .map(|file| file.tray)
        .unwrap_or_else(|_| TrayLabels {
            tooltip: "Better Screenshoot".into(),
            capture_area: "Capture Area".into(),
            capture_screen: "Capture Screen".into(),
            capture_window: "Capture Window".into(),
            history: "History".into(),
            settings: "Settings".into(),
            quit: "Quit".into(),
        })
}

fn current_locale(settings: &AppSettings) -> String {
    match settings.locale.as_str() {
        "en" | "es" | "fr" | "de" | "pt" | "it" => settings.locale.clone(),
        _ => "en".into(),
    }
}

fn build_menu(app: &AppHandle, labels: &TrayLabels) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let capture_area = MenuItem::with_id(app, "capture-area", &labels.capture_area, true, None::<&str>)?;
    let capture_screen =
        MenuItem::with_id(app, "capture-screen", &labels.capture_screen, true, None::<&str>)?;
    let capture_window =
        MenuItem::with_id(app, "capture-window", &labels.capture_window, true, None::<&str>)?;
    let history = MenuItem::with_id(app, "history", &labels.history, true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", &labels.settings, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", &labels.quit, true, None::<&str>)?;

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

    Ok(menu)
}

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let locale = {
        let state = app.state::<AppState>();
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        current_locale(&settings)
    };
    let labels = load_tray_labels(&locale);
    let menu = build_menu(app, &labels)?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().cloned().expect("tray icon"))
        .menu(&menu)
        .tooltip(&labels.tooltip)
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

#[tauri::command]
pub fn rebuild_tray_menu(app: AppHandle) -> Result<(), String> {
    let locale = {
        let state = app.state::<AppState>();
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        current_locale(&settings)
    };
    let labels = load_tray_labels(&locale);
    let menu = build_menu(&app, &labels).map_err(|e| e.to_string())?;

    let tray = app
        .tray_by_id("main-tray")
        .ok_or_else(|| "Tray icon not found".to_string())?;

    update_tray(&tray, &menu, &labels.tooltip)
}

fn update_tray(
    tray: &TrayIcon<tauri::Wry>,
    menu: &Menu<tauri::Wry>,
    tooltip: &str,
) -> Result<(), String> {
    tray.set_menu(Some(menu.clone()))
        .map_err(|e| e.to_string())?;
    tray.set_tooltip(Some(tooltip))
        .map_err(|e| e.to_string())?;
    Ok(())
}
