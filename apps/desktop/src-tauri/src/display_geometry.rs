//! Multi-monitor geometry in a single desktop coordinate space.
//!
//! On macOS tao scales each monitor's position and size by *that* monitor's backing factor,
//! the cursor by the primary monitor's factor, and window positions by the window's current
//! factor. With mixed-DPI setups (Retina laptop + 1x external display) those "physical"
//! values do not share a coordinate space, so on macOS everything here works in global
//! points. Other platforms already report one physical virtual-desktop space.

use tauri::{AppHandle, Monitor, Position, WebviewWindow};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesktopRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl DesktopRect {
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    fn intersection_area(&self, other: &DesktopRect) -> f64 {
        let width = (self.x + self.width).min(other.x + other.width) - self.x.max(other.x);
        let height = (self.y + self.height).min(other.y + other.height) - self.y.max(other.y);
        width.max(0.0) * height.max(0.0)
    }
}

fn desktop_scale(monitor: &Monitor) -> f64 {
    if cfg!(target_os = "macos") {
        monitor.scale_factor()
    } else {
        1.0
    }
}

pub fn monitor_bounds(monitor: &Monitor) -> DesktopRect {
    let scale = desktop_scale(monitor);
    DesktopRect {
        x: monitor.position().x as f64 / scale,
        y: monitor.position().y as f64 / scale,
        width: monitor.size().width as f64 / scale,
        height: monitor.size().height as f64 / scale,
    }
}

pub fn monitor_work_area(monitor: &Monitor) -> DesktopRect {
    let scale = desktop_scale(monitor);
    let work = monitor.work_area();
    DesktopRect {
        x: work.position.x as f64 / scale,
        y: work.position.y as f64 / scale,
        width: work.size.width as f64 / scale,
        height: work.size.height as f64 / scale,
    }
}

pub fn cursor_point(app: &AppHandle) -> Option<(f64, f64)> {
    let cursor = app.cursor_position().ok()?;
    let scale = if cfg!(target_os = "macos") {
        app.primary_monitor()
            .ok()
            .flatten()
            .map(|monitor| monitor.scale_factor())
            .unwrap_or(1.0)
    } else {
        1.0
    };
    Some((cursor.x / scale, cursor.y / scale))
}

pub fn monitor_under_cursor(app: &AppHandle) -> Option<Monitor> {
    let (x, y) = cursor_point(app)?;
    app.available_monitors()
        .ok()?
        .into_iter()
        .find(|monitor| monitor_bounds(monitor).contains(x, y))
}

/// Finds the monitor for a position expressed in its own monitor's scale (tray icon rects
/// on macOS) and returns it together with the position in desktop units.
pub fn locate_scaled_position(app: &AppHandle, position: Position) -> Option<(Monitor, f64, f64)> {
    let monitors = app.available_monitors().ok()?;
    match position {
        Position::Logical(point) => monitors
            .into_iter()
            .find(|monitor| monitor_bounds(monitor).contains(point.x, point.y))
            .map(|monitor| (monitor, point.x, point.y)),
        Position::Physical(point) => monitors.into_iter().find_map(|monitor| {
            let scale = desktop_scale(&monitor);
            let (x, y) = (point.x as f64 / scale, point.y as f64 / scale);
            monitor_bounds(&monitor)
                .contains(x, y)
                .then_some((monitor, x, y))
        }),
    }
}

pub fn window_bounds(window: &WebviewWindow) -> Option<DesktopRect> {
    let position = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;
    let scale = if cfg!(target_os = "macos") {
        window.scale_factor().ok()?
    } else {
        1.0
    };
    Some(DesktopRect {
        x: position.x as f64 / scale,
        y: position.y as f64 / scale,
        width: size.width as f64 / scale,
        height: size.height as f64 / scale,
    })
}

pub fn set_window_origin(window: &WebviewWindow, x: f64, y: f64) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    {
        window.set_position(tauri::LogicalPosition::new(x, y))
    }
    #[cfg(not(target_os = "macos"))]
    {
        window.set_position(tauri::PhysicalPosition::new(
            x.round() as i32,
            y.round() as i32,
        ))
    }
}

pub fn set_window_size(window: &WebviewWindow, width: f64, height: f64) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    {
        window.set_size(tauri::LogicalSize::new(width, height))
    }
    #[cfg(not(target_os = "macos"))]
    {
        window.set_size(tauri::PhysicalSize::new(
            width.round() as u32,
            height.round() as u32,
        ))
    }
}

/// Whether most of the window already sits on `monitor`.
pub fn window_is_on_monitor(window: &WebviewWindow, monitor: &Monitor) -> bool {
    let Some(window_rect) = window_bounds(window) else {
        return false;
    };
    let total = window_rect.width * window_rect.height;
    total > 0.0 && monitor_bounds(monitor).intersection_area(&window_rect) * 2.0 >= total
}

/// Origin that centers a window of `size` inside `area`, kept inside it when the window is larger.
pub fn centered_origin(size: (f64, f64), area: DesktopRect) -> (f64, f64) {
    let x = area.x + ((area.width - size.0) / 2.0).max(0.0);
    let y = area.y + ((area.height - size.1) / 2.0).max(0.0);
    (x, y)
}

/// Origin for a popover attached to a tray icon, clamped to the monitor's work area. It hangs
/// below the icon (macOS menu bar) and flips above it when there is no room (bottom taskbars).
pub fn popover_origin(anchor: DesktopRect, size: (f64, f64), area: DesktopRect) -> (f64, f64) {
    const GAP: f64 = 6.0;
    const MARGIN: f64 = 8.0;

    let min_x = area.x + MARGIN;
    let max_x = (area.x + area.width - size.0 - MARGIN).max(min_x);
    let x = (anchor.x + anchor.width / 2.0 - size.0 / 2.0).clamp(min_x, max_x);

    let below = (anchor.y + anchor.height).max(area.y) + GAP;
    let y = if below + size.1 <= area.y + area.height {
        below
    } else {
        (anchor.y.min(area.y + area.height) - size.1 - GAP).max(area.y)
    };
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f64, y: f64, width: f64, height: f64) -> DesktopRect {
        DesktopRect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn popover_hangs_below_icon_on_secondary_monitor_above_primary() {
        let area = rect(-336.0, -1415.0, 2560.0, 1415.0);
        let icon = rect(1800.0, -1440.0, 30.0, 25.0);
        let (x, y) = popover_origin(icon, (300.0, 408.0), area);
        assert_eq!(x, 1665.0);
        assert_eq!(y, -1409.0);
    }

    #[test]
    fn popover_stays_inside_work_area_near_right_edge() {
        let area = rect(0.0, 25.0, 1728.0, 1092.0);
        let icon = rect(1700.0, 0.0, 24.0, 24.0);
        let (x, _) = popover_origin(icon, (300.0, 408.0), area);
        assert_eq!(x, 1728.0 - 300.0 - 8.0);
    }

    #[test]
    fn popover_flips_above_bottom_taskbar_icon() {
        let area = rect(0.0, 0.0, 1920.0, 1040.0);
        let icon = rect(1800.0, 1045.0, 24.0, 30.0);
        let (_, y) = popover_origin(icon, (300.0, 408.0), area);
        assert_eq!(y, 1040.0 - 408.0 - 6.0);
    }

    #[test]
    fn centered_origin_centers_and_clamps() {
        let area = rect(-336.0, -1415.0, 2560.0, 1415.0);
        assert_eq!(centered_origin((980.0, 660.0), area), (454.0, -1037.5));
        assert_eq!(centered_origin((3000.0, 2000.0), area), (-336.0, -1415.0));
    }

    #[test]
    fn rect_contains_is_half_open() {
        let r = rect(0.0, 0.0, 10.0, 10.0);
        assert!(r.contains(0.0, 0.0));
        assert!(!r.contains(10.0, 5.0));
    }
}
