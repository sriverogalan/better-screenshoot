use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DisplayInfo {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
    pub is_primary: bool,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowInfo {
    pub id: u64,
    pub title: String,
    pub app_name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    pub fn validate(&self) -> bool {
        self.width > 0 && self.height > 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureImage {
    pub width: u32,
    pub height: u32,
    pub png_bytes: Vec<u8>,
    #[serde(default, skip_serializing)]
    pub rgba_bytes: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::Region;

    #[test]
    fn region_validate_requires_positive_dimensions() {
        assert!(Region { x: 0, y: 0, width: 10, height: 10 }.validate());
        assert!(!Region { x: 0, y: 0, width: 0, height: 10 }.validate());
    }
}
