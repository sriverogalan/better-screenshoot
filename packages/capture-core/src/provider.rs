use crate::error::CaptureResult;
use crate::types::{CaptureImage, DisplayInfo, Region, WindowInfo};

pub trait CaptureProvider: Send + Sync {
    fn list_displays(&self) -> CaptureResult<Vec<DisplayInfo>>;

    fn list_windows(&self) -> CaptureResult<Vec<WindowInfo>>;

    fn capture_display_rgba(&self, display_id: u32) -> CaptureResult<image::RgbaImage>;

    fn capture_display(&self, display_id: u32) -> CaptureResult<CaptureImage>;

    fn capture_window(&self, window_id: u64) -> CaptureResult<CaptureImage>;

    fn capture_region(&self, display_id: u32, region: Region) -> CaptureResult<CaptureImage>;

    fn capture_primary_display(&self) -> CaptureResult<CaptureImage> {
        let displays = self.list_displays()?;
        let primary = displays
            .iter()
            .find(|d| d.is_primary)
            .or(displays.first())
            .ok_or(crate::error::CaptureError::NoDisplays)?;
        self.capture_display(primary.id)
    }
}
