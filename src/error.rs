use std::error::Error;
use tokio::io;
use crate::logging::log_error;

// Define custom error type for the application.
// This allows for more granular error handling and reporting.
#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Watch(notify::Error),
    Processing(String),
}

// Implement `From` trait for common error types to convert them into `AppError`.
impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        log_error("IO Error occurred", &err);
        AppError::Io(err)
    }
}

impl From<notify::Error> for AppError {
    fn from(err: notify::Error) -> Self {
        log_error("File watch error occurred", &err);
        AppError::Watch(err)
    }
}

// Implement `Display` and `Error` traits for `AppError` to make it a proper error type.
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "IO error: {}", err),
            AppError::Watch(err) => write!(f, "File watch error: {}", err),
            AppError::Processing(msg) => write!(f, "Processing error: {}", msg),
        }
    }
}

impl Error for AppError {}

impl AppError {
    /// Create a new processing error with logging
    pub fn processing_error(msg: String) -> Self {
        log_error("Processing error", &msg);
        AppError::Processing(msg)
    }
    
    /// Log this error with additional context
    pub fn log_with_context(&self, context: &str) {
        log_error(context, self);
    }
}