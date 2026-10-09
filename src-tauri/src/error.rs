use serde::Serialize;

/// All errors that can cross the IPC boundary to the frontend.
///
/// Every variant carries enough context for the UI to explain the failure in
/// plain language instead of showing a raw error string.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("windows api error: {0}")]
    WindowsApi(String),

    #[error("unsupported on this platform or device: {0}")]
    Unsupported(String),

    #[error("audio error: {0}")]
    Audio(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),
}

/// Stable machine-readable error code, used by the frontend to pick an icon /
/// recovery action without string-matching the human message.
impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "database_error",
            AppError::Io(_) => "io_error",
            AppError::NotFound(_) => "not_found",
            AppError::InvalidInput(_) => "invalid_input",
            AppError::WindowsApi(_) => "windows_api_error",
            AppError::Unsupported(_) => "unsupported",
            AppError::Audio(_) => "audio_error",
            AppError::PermissionDenied(_) => "permission_denied",
        }
    }
}

#[derive(Serialize)]
pub struct ErrorPayload {
    pub code: &'static str,
    pub message: String,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ErrorPayload {
            code: self.code(),
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}

pub type AppResult<T> = Result<T, AppError>;
