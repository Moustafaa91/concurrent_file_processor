use flexi_logger::{Duplicate, FileSpec, FlexiLoggerError, Logger};
use log::{info, error, warn, debug};
use crate::config::LoggingConfig;

pub fn init_logging(config: &LoggingConfig) -> Result<(), FlexiLoggerError> {
    let mut logger = Logger::try_with_str(&config.level)?
        .log_to_file(FileSpec::default()
            .directory(&config.log_dir)
            .basename(&config.log_basename));
    
    if config.duplicate_to_stdout {
        logger = logger.duplicate_to_stdout(Duplicate::Info);
    }
    
    logger.start()?;
    
    info!("Logging system initialized successfully");
    Ok(())
}

/// Log an error with additional context
pub fn log_error<E: std::fmt::Display>(context: &str, error: &E) {
    error!("{} | {}", context, error);
}

/// Log a warning with additional context
pub fn log_warning(context: &str, message: &str) {
    warn!("{} | {}", context, message);
}

/// Log debug information
pub fn log_debug(context: &str, message: &str) {
    debug!("{} | {}", context, message);
}

/// Log info message
pub fn log_info(context: &str, message: &str) {
    info!("{} | {}", context, message);
}

/// Log info message without context (for simple messages)
pub fn log_info_simple(message: &str) {
    info!("{}", message);
}

/// Log error message without context (for simple error messages)
pub fn log_error_simple(message: &str) {
    error!("{}", message);
}
