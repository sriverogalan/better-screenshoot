use crate::provider::CaptureProvider;

mod xcap_helpers;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

pub fn create_provider() -> Box<dyn CaptureProvider> {
    #[cfg(target_os = "macos")]
    {
        return Box::new(macos::MacOSProvider::new());
    }
    #[cfg(target_os = "windows")]
    {
        return Box::new(windows::WindowsProvider::new());
    }
    #[cfg(target_os = "linux")]
    {
        return Box::new(linux::LinuxProvider::new());
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        struct Unsupported;
        impl CaptureProvider for Unsupported {
            fn list_displays(&self) -> crate::error::CaptureResult<Vec<crate::types::DisplayInfo>> {
                Err(crate::error::CaptureError::PlatformNotSupported(
                    "unsupported operating system".into(),
                ))
            }
            fn list_windows(&self) -> crate::error::CaptureResult<Vec<crate::types::WindowInfo>> {
                Err(crate::error::CaptureError::PlatformNotSupported(
                    "unsupported operating system".into(),
                ))
            }
            fn capture_display_rgba(
                &self,
                _display_id: u32,
            ) -> crate::error::CaptureResult<image::RgbaImage> {
                Err(crate::error::CaptureError::PlatformNotSupported(
                    "unsupported operating system".into(),
                ))
            }
            fn capture_display(
                &self,
                _display_id: u32,
            ) -> crate::error::CaptureResult<crate::types::CaptureImage> {
                Err(crate::error::CaptureError::PlatformNotSupported(
                    "unsupported operating system".into(),
                ))
            }
            fn capture_window(
                &self,
                _window_id: u64,
            ) -> crate::error::CaptureResult<crate::types::CaptureImage> {
                Err(crate::error::CaptureError::PlatformNotSupported(
                    "unsupported operating system".into(),
                ))
            }
            fn capture_region(
                &self,
                _display_id: u32,
                _region: crate::types::Region,
            ) -> crate::error::CaptureResult<crate::types::CaptureImage> {
                Err(crate::error::CaptureError::PlatformNotSupported(
                    "unsupported operating system".into(),
                ))
            }
        }
        Box::new(Unsupported)
    }
}
