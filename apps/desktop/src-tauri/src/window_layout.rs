use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{
    AppHandle, LogicalSize, Manager, Monitor, PhysicalPosition, PhysicalSize, Position, Size,
    WebviewWindow,
};

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
    if let Some(monitor) = monitor_under_cursor(window) {
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

/// Monitor containing the cursor (the one the user is using right now).
fn monitor_under_cursor(window: &WebviewWindow) -> Option<Monitor> {
    let app = window.app_handle();
    let cursor = app.cursor_position().ok()?;
    let monitors = window.available_monitors().ok()?;
    monitors.into_iter().find(|monitor| {
        let pos = monitor.position();
        let size = monitor.size();
        let x = cursor.x as i32;
        let y = cursor.y as i32;
        x >= pos.x && x < pos.x + size.width as i32 && y >= pos.y && y < pos.y + size.height as i32
    })
}

/// Moves the editor to the monitor where the cursor is so it appears where the user is working.
/// Essential on multi-monitor: otherwise the window opens fullscreen on the wrong monitor.
pub fn move_editor_to_active_monitor(window: &WebviewWindow) -> Result<(), String> {
    let Some(monitor) = monitor_under_cursor(window) else {
        return Ok(());
    };

    let pos = monitor.position();
    crate::app_trace!(
        "move_editor_to_active_monitor: monitor en ({}, {})",
        pos.x,
        pos.y
    );
    window
        .set_position(Position::Physical(PhysicalPosition { x: pos.x, y: pos.y }))
        .map_err(|e| e.to_string())?;
    Ok(())
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
    let work = monitor.work_area();
    window
        .set_position(Position::Physical(PhysicalPosition {
            x: work.position.x,
            y: work.position.y,
        }))
        .map_err(|e| e.to_string())?;
    window
        .set_size(Size::Physical(PhysicalSize {
            width: work.size.width,
            height: work.size.height,
        }))
        .map_err(|e| e.to_string())?;
    Ok(())
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
    Ok(())
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
