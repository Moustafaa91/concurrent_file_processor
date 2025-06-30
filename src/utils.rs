use crate::error::AppError;
use crate::logging::{log_info, log_error, log_error_simple, log_debug};
use crate::config::ProcessingConfig;
use sha2::{Sha256, Digest};
use hex;

use tokio::io::AsyncWriteExt;
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
    time::{sleep, Duration},
};

use std::path::{Path, PathBuf};

// File size formatting constants (these are not configuration, they're just formatting helpers)
const KB: u64 = 1024;
const MB: u64 = KB * 1024;
const GB: u64 = MB * 1024;

/// Hash file content using SHA256
pub fn hash_file_content(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Check if a file is available for processing (not locked by another process)
pub async fn is_file_available(file_path: &Path, config: &ProcessingConfig) -> bool {
    match File::open(file_path).await {
        Ok(_) => true,
        Err(e) => {
            if e.raw_os_error() == Some(config.file_locked_error_code) {
                log_debug("File is temporarily locked", &format!("{}", file_path.display()));
            } else {
                log_debug("File not available", &format!("{}: {}", file_path.display(), e));
            }
            false
        }
    }
}

/// Wait for file availability with exponential backoff
pub async fn wait_for_file_availability(
    file_path: &Path, 
    config: &ProcessingConfig
) -> Result<(), AppError> {
    let mut retry_count = 0;
    let mut delay = Duration::from_millis(config.initial_retry_delay_ms);
    
    while retry_count < config.max_retries {
        if is_file_available(file_path, config).await {
            return Ok(());
        }
        
        log_info("File not available, retrying", &format!("{} (attempt {}/{})", 
            file_path.display(), retry_count + 1, config.max_retries));
        
        sleep(delay).await;
        retry_count += 1;
        
        // Exponential backoff: double the delay for next retry
        delay = Duration::min(delay * 2, Duration::from_secs(config.max_retry_delay_sec));
    }
    
    log_error("File still not available after retries", &format!("{}", file_path.display()));
    Err(AppError::Processing(format!("File {} is not available after retries", file_path.display())))
}

/// Read file content asynchronously
pub async fn read_file_content(file_path: &Path) -> Result<Vec<u8>, AppError> {
    let mut file = File::open(file_path).await.map_err(|e| {
        log_error("Failed to open file", &format!("{}: {}", file_path.display(), e));
        AppError::Io(e)
    })?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await.map_err(|e| {
        log_error("Failed to read file", &format!("{}: {}", file_path.display(), e));
        AppError::Io(e)
    })?;
    
    log_info("Read bytes from file", &format!("{} bytes from {}", buffer.len(), file_path.display()));
    Ok(buffer)
}

/// Extract file name from path
pub fn extract_file_name(file_path: &Path) -> Result<String, AppError> {
    Ok(file_path.file_name()
        .ok_or_else(|| {
            let msg = format!("Could not get file name for {}", file_path.display());
            log_error_simple(&msg);
            AppError::Processing(msg)
        })?
        .to_string_lossy()
        .into_owned())
}

/// Write processed data to output file
pub async fn write_processed_data(
    file_name: &str, 
    processed_data: &str, 
    output_dir: &Path,
    output_extension: &str
) -> Result<PathBuf, AppError> {
    let output_file_name = format!("{}{}", 
        Path::new(file_name).file_stem().unwrap().to_string_lossy(),
        output_extension
    );
    let output_path = output_dir.join(&output_file_name);
    
    log_info("Writing processed output to", &format!("{}", output_path.display()));

    let mut output_file = File::create(&output_path).await.map_err(|e| {
        log_error("Failed to create output file", &format!("{}: {}", output_path.display(), e));
        AppError::Io(e)
    })?;
    
    output_file.write_all(processed_data.as_bytes()).await.map_err(|e| {
        log_error("Failed to write to output file", &format!("{}: {}", output_path.display(), e));
        AppError::Io(e)
    })?;
    
    log_info("Wrote processed data to", &format!("{}", output_path.display()));
    Ok(output_path)
}

/// Remove original file after processing
pub async fn remove_original_file(file_path: &Path) -> Result<(), AppError> {
    fs::remove_file(file_path).await.map_err(|e| {
        log_error("Failed to remove original file", &format!("{}: {}", file_path.display(), e));
        AppError::Io(e)
    })?;
    
    log_info("Original file removed", &format!("{}", file_path.display()));
    Ok(())
}

/// Scan input directory for files to process
pub async fn scan_input_directory(input_dir: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut existing_files = Vec::new();
    let mut read_dir = fs::read_dir(input_dir).await.map_err(|e| {
        log_error("Failed to read input directory for initial scan", &e);
        AppError::Io(e)
    })?;
    
    while let Some(entry) = read_dir.next_entry().await.map_err(|e| {
        log_error("Failed to read directory entry during initial scan", &e);
        AppError::Io(e)
    })? {
        let path = entry.path();
        if path.is_file() {
            existing_files.push(path);
        }
    }
    
    Ok(existing_files)
}

/// Create output directory if it doesn't exist
pub async fn ensure_output_directory(output_dir: &Path) -> Result<(), AppError> {
    if !output_dir.exists() {
        log_info("Creating output directory", &format!("{}", output_dir.display()));
        fs::create_dir_all(output_dir).await.map_err(|e| {
            log_error("Failed to create output directory", &format!("{}: {}", output_dir.display(), e));
            AppError::Io(e)
        })?;
    }
    Ok(())
}

/// Validate file path and check if it's a regular file
pub fn validate_file_path(file_path: &Path) -> Result<(), AppError> {
    if !file_path.exists() {
        return Err(AppError::Processing(format!("File does not exist: {}", file_path.display())));
    }
    
    if !file_path.is_file() {
        return Err(AppError::Processing(format!("Path is not a file: {}", file_path.display())));
    }
    
    Ok(())
}

/// Get file size in bytes
pub async fn get_file_size(file_path: &Path) -> Result<u64, AppError> {
    let metadata = fs::metadata(file_path).await.map_err(|e| {
        log_error("Failed to get file metadata", &format!("{}: {}", file_path.display(), e));
        AppError::Io(e)
    })?;
    
    Ok(metadata.len())
}

/// Format file size in human-readable format
pub fn format_file_size(bytes: u64) -> String {
    match bytes {
        0..KB => format!("{} B", bytes),
        KB..MB => format!("{:.1} KB", bytes as f64 / KB as f64),
        MB..GB => format!("{:.1} MB", bytes as f64 / MB as f64),
        _ => format!("{:.1} GB", bytes as f64 / GB as f64),
    }
}

/// Setup input and output directories, creating them if they don't exist
pub async fn setup_directories(input_dir: &PathBuf, output_dir: &PathBuf) -> Result<(), AppError> {
    // Ensure input directory exists
    fs::create_dir_all(input_dir).await.map_err(|e| {
        log_error("Failed to create input directory", &e);
        AppError::Io(e)
    })?;
    
    // Ensure output directory exists
    ensure_output_directory(output_dir).await?;

    log_info("Input directory set to", &format!("{}", input_dir.display()));
    log_info("Output directory set to", &format!("{}", output_dir.display()));

    Ok(())
} 