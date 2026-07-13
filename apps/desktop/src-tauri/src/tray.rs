use serde::Deserialize;
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, LogicalPosition, Manager, WebviewWindow,
};

use crate::state::{AppSettings, AppState};

#[derive(Debug, Deserialize)]
struct TrayLocaleFile {
    tray: TrayLabels,
}

#[derive(Debug, Deserialize)]
struct TrayLabels {
    tooltip: String,
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
        })
}

fn current_locale(settings: &AppSettings) -> String {
    match settings.locale.as_str() {
        "en" | "es" | "fr" | "de" | "pt" | "it" => settings.locale.clone(),
        _ => "en".into(),
    }
}

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let locale = {
        let state = app.state::<AppState>();
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        current_locale(&settings)
    };
    let labels = load_tray_labels(&locale);

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().cloned().expect("tray icon"))
        .tooltip(&labels.tooltip)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left | MouseButton::Right,
                button_state: MouseButtonState::Up,
                position,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(menubar) = app.get_webview_window("menubar") {
                    toggle_menubar_panel(&menubar, position.x, position.y);
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn toggle_menubar_panel(menubar: &WebviewWindow, tray_x: f64, tray_y: f64) {
    if menubar.is_visible().unwrap_or(false) {
        let _ = menubar.hide();
        return;
    }

    let scale = menubar
        .current_monitor()
        .ok()
        .flatten()
        .map(|m| m.scale_factor())
        .unwrap_or(1.0);

    let (window_width, window_height) = menubar
        .outer_size()
        .map(|size| (size.width as f64 / scale, size.height as f64 / scale))
        .unwrap_or((300.0, 400.0));

    let (logical_x, logical_y) =
        calculate_tray_popup_position(tray_x, tray_y, scale, window_width, window_height);

    let _ = menubar.set_position(LogicalPosition::new(logical_x, logical_y));
    let _ = menubar.show();
    let _ = menubar.set_focus();
}

#[tauri::command]
pub fn update_tray_tooltip(app: AppHandle) -> Result<(), String> {
    let locale = {
        let state = app.state::<AppState>();
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        current_locale(&settings)
    };
    let labels = load_tray_labels(&locale);

    let tray = app
        .tray_by_id("main-tray")
        .ok_or_else(|| "Tray icon not found".to_string())?;

    tray.set_tooltip(Some(&labels.tooltip))
        .map_err(|e| e.to_string())
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

    #[test]
    fn tray_labels_provide_tooltip_for_each_locale() {
        for locale in ["en", "es", "fr", "de", "pt", "it", "unknown"] {
            let labels = load_tray_labels(locale);
            assert!(!labels.tooltip.is_empty());
        }
    }
}
