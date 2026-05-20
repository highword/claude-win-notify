use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Windows(windows::core::Error),
    StdinIsTerminal,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Json(e) => write!(f, "JSON error: {}", e),
            AppError::Windows(e) => write!(f, "Windows error: {}", e),
            AppError::StdinIsTerminal => write!(f, "stdin is a terminal (expected piped input)"),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e)
    }
}

impl From<windows::core::Error> for AppError {
    fn from(e: windows::core::Error) -> Self {
        AppError::Windows(e)
    }
}
