use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, LogicalSize, Manager, Monitor, Size, WebviewWindow};

use crate::display_geometry;

static MAIN_RESTORING: AtomicBool = AtomicBool::new(false);
static MAIN_EDITOR_MODE: AtomicBool = AtomicBool::new(false);

const EDITOR_WINDOW_WIDTH: f64 = 900.0;
const EDITOR_WINDOW_HEIGHT: f64 = 700.0;

pub fn reset_editor_fullscreen_state(window: &WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let _ = window.set_simple_fullscreen(false);
    let _ = window.set_fullscreen(false);
    Ok(())
}

/// Leaves the editor in a neutral state before presenting: no overlay, no always-on-top,
/// and out of any previous fullscreen mode.
pub fn reset_editor_presentation_state(window: &WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        let _ = window.set_visible_on_all_workspaces(false);
        let _ = window.set_always_on_top(false);
        let _ = window.set_simple_fullscreen(false);
    }
    let _ = window.set_fullscreen(false);
}

fn resolve_monitor(window: &WebviewWindow) -> Result<Monitor, String> {
    if let Some(monitor) = display_geometry::monitor_under_cursor(window.app_handle()) {
        return Ok(monitor);
    }

    if let Ok(Some(monitor)) = window.current_monitor() {
        return Ok(monitor);
    }

    window
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| crate::errors::app_error("noMonitorFound"))
}

/// Centers the window on the monitor under the cursor unless it is already there, so it
/// appears on the screen the user is working on.
pub fn move_to_active_monitor(window: &WebviewWindow) -> Result<(), String> {
    let Some(monitor) = display_geometry::monitor_under_cursor(window.app_handle()) else {
        return Ok(());
    };
    if display_geometry::window_is_on_monitor(window, &monitor) {
        return Ok(());
    }

    let Some(bounds) = display_geometry::window_bounds(window) else {
        return Ok(());
    };
    let area = display_geometry::monitor_work_area(&monitor);
    let (x, y) = display_geometry::centered_origin((bounds.width, bounds.height), area);
    crate::app_trace!("move_to_active_monitor: {} -> ({x}, {y})", window.label());
    display_geometry::set_window_origin(window, x, y).map_err(|e| e.to_string())
}

/// Pantalla completa del editor tras captura.
/// macOS: `simple_fullscreen` cubre el monitor sin crear un Space nuevo ni dejar el WKWebView en negro.
pub fn enter_editor_capture_fullscreen(window: &WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        window
            .set_simple_fullscreen(true)
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        window.set_fullscreen(true).map_err(|e| e.to_string())
    }
}

pub fn fill_work_area(window: &WebviewWindow) -> Result<(), String> {
    let monitor = resolve_monitor(window)?;
    let work = display_geometry::monitor_work_area(&monitor);
    display_geometry::set_window_origin(window, work.x, work.y).map_err(|e| e.to_string())?;
    display_geometry::set_window_size(window, work.width, work.height).map_err(|e| e.to_string())
}

pub fn restore_windowed_editor(window: &WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let _ = window.set_visible_on_all_workspaces(false);
        let _ = window.set_always_on_top(false);
        let _ = window.set_decorations(false);
    }
    window
        .set_size(Size::Logical(LogicalSize {
            width: EDITOR_WINDOW_WIDTH,
            height: EDITOR_WINDOW_HEIGHT,
        }))
        .map_err(|e| e.to_string())?;
    window.center().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn reset_editor_window_layout(app: AppHandle) -> Result<(), String> {
    let editor = app
        .get_webview_window("editor")
        .ok_or_else(|| crate::errors::app_error("editorNotFound"))?;
    reset_editor_fullscreen_state(&editor)?;
    restore_windowed_editor(&editor)
}

pub fn prepare_main_hub_window(window: &WebviewWindow) -> Result<(), String> {
    if MAIN_RESTORING.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    let result = prepare_main_hub_window_inner(window);
    MAIN_RESTORING.store(false, Ordering::SeqCst);
    result
}

fn prepare_main_hub_window_inner(window: &WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let _ = window.set_simple_fullscreen(false);
    let _ = window.set_fullscreen(false);
    // The main window is reused as the editor surface, which turns decorations off
    // (see present_editor_window). Restore them whenever the hub is shown again,
    // otherwise the native traffic-light buttons stay missing for the rest of the session.
    let _ = window.set_decorations(true);
    // Toggling decorations rebuilds the macOS style mask and drops the full-size content
    // view, so the transparent overlay title bar has to be re-applied afterwards.
    #[cfg(target_os = "macos")]
    let _ = window.set_title_bar_style(tauri::TitleBarStyle::Overlay);
    #[cfg(target_os = "macos")]
    extend_webview_under_titlebar(window);
    // A window the user can already see stays where they put it; a hidden one reappears
    // on the screen they are working on.
    if !window.is_visible().unwrap_or(false) {
        move_to_active_monitor(window)?;
    }
    Ok(())
}

/// With the overlay title bar the web view already spans the whole window, but WebKit
/// insets the page by the title bar height on its own unless automatic insets are off.
#[cfg(target_os = "macos")]
fn extend_webview_under_titlebar(window: &WebviewWindow) {
    let _ = window.with_webview(|webview| unsafe {
        use objc2::runtime::AnyObject;
        use objc2::{msg_send, sel};

        let wk_webview = webview.inner() as *mut AnyObject;
        let supported: bool = msg_send![
            wk_webview,
            respondsToSelector: sel!(_setAutomaticallyAdjustsContentInsets:)
        ];
        if supported {
            let _: () = msg_send![wk_webview, _setAutomaticallyAdjustsContentInsets: false];
        }
    });
}

#[tauri::command]
pub fn reset_main_window_layout(app: AppHandle) -> Result<(), String> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| crate::errors::app_error("mainWindowNotFound"))?;
    prepare_main_hub_window(&main)
}

pub fn set_main_editor_mode(active: bool) {
    MAIN_EDITOR_MODE.store(active, Ordering::SeqCst);
}

pub fn is_main_editor_mode() -> bool {
    MAIN_EDITOR_MODE.load(Ordering::SeqCst)
}

#[tauri::command]
pub fn exit_main_editor_mode(app: AppHandle) -> Result<(), String> {
    set_main_editor_mode(false);
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| crate::errors::app_error("mainWindowNotFound"))?;
    reset_editor_fullscreen_state(&main)?;
    prepare_main_hub_window(&main)
}
