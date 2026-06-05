use xcap::{Monitor, Window};

use crate::encode::encode_png;
use crate::error::{CaptureError, CaptureResult};
use crate::platform::xcap_helpers::region_to_u32;
use crate::provider::CaptureProvider;
use crate::types::{CaptureImage, DisplayInfo, Region, WindowInfo};

pub struct MacOSProvider;

impl MacOSProvider {
    pub fn new() -> Self {
        Self
    }

    fn find_monitor(id: u32) -> CaptureResult<Monitor> {
        let monitors = Monitor::all().map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        monitors
            .into_iter()
            .find(|m| m.id().unwrap_or(0) as u32 == id)
            .ok_or(CaptureError::DisplayNotFound(id))
    }

    fn find_window(id: u64) -> CaptureResult<Window> {
        let windows = Window::all().map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        windows
            .into_iter()
            .find(|w| w.id().unwrap_or(0) as u64 == id)
            .ok_or(CaptureError::WindowNotFound(id))
    }
}

impl CaptureProvider for MacOSProvider {
    fn list_displays(&self) -> CaptureResult<Vec<DisplayInfo>> {
        let monitors = Monitor::all().map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        if monitors.is_empty() {
            return Err(CaptureError::PermissionDenied(
                "macOS returned no displays. Enable Screen Recording for Better Screenshoot in System Settings → Privacy & Security.".into(),
            ));
        }

        Ok(monitors
            .into_iter()
            .map(|monitor| DisplayInfo {
                id: monitor.id().unwrap_or(0) as u32,
                name: monitor.friendly_name().unwrap_or_else(|_| "Display".into()),
                width: monitor.width().unwrap_or(0),
                height: monitor.height().unwrap_or(0),
                scale_factor: monitor.scale_factor().unwrap_or(1.0) as f64,
                is_primary: monitor.is_primary().unwrap_or(false),
                x: monitor.x().unwrap_or(0),
                y: monitor.y().unwrap_or(0),
            })
            .collect())
    }

    fn list_windows(&self) -> CaptureResult<Vec<WindowInfo>> {
        let windows = Window::all().map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        Ok(windows
            .into_iter()
            .filter(|w| w.is_minimized().unwrap_or(true) == false)
            .map(|window| WindowInfo {
                id: window.id().unwrap_or(0) as u64,
                title: window.title().unwrap_or_default(),
                app_name: window.app_name().unwrap_or_default(),
                width: window.width().unwrap_or(0),
                height: window.height().unwrap_or(0),
                x: window.x().unwrap_or(0),
                y: window.y().unwrap_or(0),
            })
            .collect())
    }

    fn capture_display_rgba(&self, display_id: u32) -> CaptureResult<image::RgbaImage> {
        let monitor = Self::find_monitor(display_id)?;
        monitor
            .capture_image()
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))
    }

    fn capture_display(&self, display_id: u32) -> CaptureResult<CaptureImage> {
        let image = self.capture_display_rgba(display_id)?;
        encode_png(image)
    }

    fn capture_window(&self, window_id: u64) -> CaptureResult<CaptureImage> {
        let window = Self::find_window(window_id)?;
        let image = window
            .capture_image()
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        encode_png(image)
    }

    fn capture_region(&self, display_id: u32, region: Region) -> CaptureResult<CaptureImage> {
        if !region.validate() {
            return Err(CaptureError::InvalidRegion {
                message: "width and height must be greater than zero".into(),
            });
        }
        let monitor = Self::find_monitor(display_id)?;
        let (x, y, width, height) = region_to_u32(region)?;
        let image = monitor
            .capture_region(x, y, width, height)
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        encode_png(image)
    }
}

