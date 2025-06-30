use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::AppError;
use crate::logging::{log_info, log_error};

/// Application configuration loaded from TOML file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// File processing configuration
    pub processing: ProcessingConfig,
    /// Directory configuration
    pub directories: DirectoryConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// File watcher configuration
    pub watcher: WatcherConfig,
}

/// File processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    /// Maximum number of retries for file operations
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    pub initial_retry_delay_ms: u64,
    /// Maximum retry delay in seconds
    pub max_retry_delay_sec: u64,
    /// Output file extension
    pub output_extension: String,
    /// File locked error code for Windows
    pub file_locked_error_code: i32,
}

/// Directory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryConfig {
    /// Input directory path
    pub input_dir: String,
    /// Output directory path
    pub output_dir: String,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (debug, info, warn, error)
    pub level: String,
    /// Log directory path
    pub log_dir: String,
    /// Log file basename
    pub log_basename: String,
    /// Whether to duplicate logs to stdout
    pub duplicate_to_stdout: bool,
}

/// File watcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    /// Channel buffer size for watcher events
    pub channel_buffer_size: usize,
    /// Delay in milliseconds before processing new files
    pub processing_delay_ms: u64,
    /// Whether to watch directories recursively
    pub recursive: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            processing: ProcessingConfig::default(),
            directories: DirectoryConfig::default(),
            logging: LoggingConfig::default(),
            watcher: WatcherConfig::default(),
        }
    }
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            max_retries: 10,
            initial_retry_delay_ms: 100,
            max_retry_delay_sec: 2,
            output_extension: ".processed.txt".to_string(),
            file_locked_error_code: 32,
        }
    }
}

impl Default for DirectoryConfig {
    fn default() -> Self {
        Self {
            input_dir: "./input_files".to_string(),
            output_dir: "./output_files".to_string(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            log_dir: "logs".to_string(),
            log_basename: "app_log".to_string(),
            duplicate_to_stdout: true,
        }
    }
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            channel_buffer_size: 32,
            processing_delay_ms: 50,
            recursive: true,
        }
    }
}

impl AppConfig {
    /// Load configuration from a TOML file
    pub fn from_file(path: &PathBuf) -> Result<Self, AppError> {
        let config_content = std::fs::read_to_string(path).map_err(|e| {
            log_error("Failed to read config file", &format!("{}: {}", path.display(), e));
            AppError::Io(e)
        })?;

        let config: AppConfig = toml::from_str(&config_content).map_err(|e| {
            log_error("Failed to parse config file", &format!("{}: {}", path.display(), e));
            AppError::Processing(format!("Invalid TOML configuration: {}", e))
        })?;

        log_info("Configuration loaded successfully", &format!("from {}", path.display()));
        Ok(config)
    }

    /// Load configuration from default location or create default config
    pub fn load_or_default() -> Result<Self, AppError> {
        let config_path = PathBuf::from("config.toml");
        
        if config_path.exists() {
            Self::from_file(&config_path)
        } else {
            log_info("No config file found", "Creating default configuration");
            let config = AppConfig::default();
            config.save_to_file(&config_path)?;
            Ok(config)
        }
    }

    /// Save configuration to a TOML file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), AppError> {
        let config_content = toml::to_string_pretty(self).map_err(|e| {
            log_error("Failed to serialize config", &e);
            AppError::Processing(format!("Failed to serialize configuration: {}", e))
        })?;

        std::fs::write(path, config_content).map_err(|e| {
            log_error("Failed to write config file", &format!("{}: {}", path.display(), e));
            AppError::Io(e)
        })?;

        log_info("Configuration saved", &format!("to {}", path.display()));
        Ok(())
    }

    /// Get input directory as PathBuf
    pub fn input_dir(&self) -> PathBuf {
        PathBuf::from(&self.directories.input_dir)
    }

    /// Get output directory as PathBuf
    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.directories.output_dir)
    }

    /// Get log directory as PathBuf
    pub fn log_dir(&self) -> PathBuf {
        PathBuf::from(&self.logging.log_dir)
    }
} 