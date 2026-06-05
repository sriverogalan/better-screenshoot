pub mod encode;
pub mod error;
pub mod platform;
pub mod provider;
pub mod types;

pub use error::{CaptureError, CaptureResult};
pub use platform::create_provider;
pub use provider::CaptureProvider;
pub use types::{CaptureImage, DisplayInfo, Region, WindowInfo};
