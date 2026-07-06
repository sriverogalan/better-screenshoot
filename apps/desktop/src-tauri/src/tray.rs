use serde::Deserialize;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, LogicalPosition, Manager,
};

use crate::shortcuts::handle_hotkey_action;
use crate::state::{AppSettings, AppState};

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
            quit: "Quit".into(),
        })
}

fn current_locale(settings: &AppSettings) -> String {
    match settings.locale.as_str() {
        "en" | "es" | "fr" | "de" | "pt" | "it" => settings.locale.clone(),
        _ => "en".into(),
    }
}

fn build_menu(
    app: &AppHandle,
    labels: &TrayLabels,
) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let capture_area = MenuItem::with_id(
        app,
        "capture-area",
        &labels.capture_area,
        true,
        None::<&str>,
    )?;
    let capture_screen = MenuItem::with_id(
        app,
        "capture-screen",
        &labels.capture_screen,
        true,
        None::<&str>,
    )?;
    let capture_window = MenuItem::with_id(
        app,
        "capture-window",
        &labels.capture_window,
        true,
        None::<&str>,
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", &labels.quit, true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &capture_area,
            &capture_screen,
            &capture_window,
            &separator,
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
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                position,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(menubar) = app.get_webview_window("menubar") {
                    let scale = menubar
                        .current_monitor()
                        .ok()
                        .flatten()
                        .map(|m| m.scale_factor())
                        .unwrap_or(1.0);

                    let (logical_x, logical_y) =
                        calculate_tray_popup_position(position.x, position.y, scale, 300.0, 408.0);

                    let _ = menubar.set_position(LogicalPosition::new(logical_x, logical_y));
                    let _ = menubar.show();
                    let _ = menubar.set_focus();
                }
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

fn calculate_tray_popup_position(
    physical_x: f64,
    physical_y: f64,
    scale: f64,
    window_width: f64,
    window_height: f64,
) -> (f64, f64) {
    let logical_x = physical_x / scale - window_width / 2.0;
    let logical_y = (physical_y / scale - window_height).max(0.0);
    (logical_x, logical_y)
}

fn update_tray(
    tray: &TrayIcon<tauri::Wry>,
    menu: &Menu<tauri::Wry>,
    tooltip: &str,
) -> Result<(), String> {
    tray.set_menu(Some(menu.clone()))
        .map_err(|e| e.to_string())?;
    tray.set_tooltip(Some(tooltip)).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_centers_window_on_tray_icon() {
        let (x, y) = calculate_tray_popup_position(800.0, 50.0, 2.0, 300.0, 408.0);
        // 800.0 / 2.0 - 300.0 / 2.0 = 400.0 - 150.0 = 250.0
        assert_eq!(x, 250.0);
        // (50.0 / 2.0 - 408.0).max(0.0) = (25.0 - 408.0).max(0.0) = 0.0
        assert_eq!(y, 0.0);
    }

    #[test]
    fn y_coordinate_clamps_to_zero() {
        let (_, y) = calculate_tray_popup_position(400.0, 10.0, 1.0, 300.0, 408.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn normal_position_below_tray_icon() {
        let (x, y) = calculate_tray_popup_position(1000.0, 1080.0, 1.0, 300.0, 408.0);
        // 1000.0 / 1.0 - 300.0 / 2.0 = 850.0
        assert_eq!(x, 850.0);
        // (1080.0 / 1.0 - 408.0).max(0.0) = 672.0
        assert_eq!(y, 672.0);
    }
}
