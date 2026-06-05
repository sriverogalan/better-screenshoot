use crate::error::{CaptureError, CaptureResult};
use crate::types::Region;

pub fn region_to_u32(region: Region) -> CaptureResult<(u32, u32, u32, u32)> {
    if region.x < 0 || region.y < 0 {
        return Err(CaptureError::InvalidRegion {
            message: "x and y must be non-negative for xcap".into(),
        });
    }
    Ok((
        region.x as u32,
        region.y as u32,
        region.width,
        region.height,
    ))
}
