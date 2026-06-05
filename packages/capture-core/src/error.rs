use thiserror::Error;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("no displays found")]
    NoDisplays,

    #[error("no windows found")]
    NoWindows,

    #[error("display not found: {0}")]
    DisplayNotFound(u32),

    #[error("window not found: {0}")]
    WindowNotFound(u64),

    #[error("invalid region: {message}")]
    InvalidRegion { message: String },

    #[error("platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("capture failed: {0}")]
    CaptureFailed(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type CaptureResult<T> = Result<T, CaptureError>;
