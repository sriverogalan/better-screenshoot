use serde::Deserialize;
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Rect, WebviewWindow,
};

use crate::display_geometry::{self, DesktopRect};
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
                rect,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(menubar) = app.get_webview_window("menubar") {
                    toggle_menubar_panel(&menubar, rect);
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn toggle_menubar_panel(menubar: &WebviewWindow, icon_rect: Rect) {
    if menubar.is_visible().unwrap_or(false) {
        let _ = menubar.hide();
        return;
    }

    if let Some(origin) = menubar_panel_origin(menubar, icon_rect) {
        let _ = display_geometry::set_window_origin(menubar, origin.0, origin.1);
    }
    let _ = menubar.show();
    let _ = menubar.set_focus();
}

/// Places the panel under the icon on the screen whose menu bar was clicked, not on the
/// screen the panel was last shown on.
fn menubar_panel_origin(menubar: &WebviewWindow, icon_rect: Rect) -> Option<(f64, f64)> {
    let app = menubar.app_handle();
    let (monitor, x, y) = display_geometry::locate_scaled_position(app, icon_rect.position)?;
    let icon_scale = if cfg!(target_os = "macos") {
        monitor.scale_factor()
    } else {
        1.0
    };
    let icon_size = icon_rect.size.to_logical::<f64>(icon_scale);
    let anchor = DesktopRect {
        x,
        y,
        width: icon_size.width,
        height: icon_size.height,
    };
    let panel = display_geometry::window_bounds(menubar)?;
    Some(display_geometry::popover_origin(
        anchor,
        (panel.width, panel.height),
        display_geometry::monitor_work_area(&monitor),
    ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_labels_provide_tooltip_for_each_locale() {
        for locale in ["en", "es", "fr", "de", "pt", "it", "unknown"] {
            let labels = load_tray_labels(locale);
            assert!(!labels.tooltip.is_empty());
        }
    }
}
