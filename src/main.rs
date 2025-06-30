use std::path::PathBuf;
use tokio::sync::mpsc;
use crate::logging::{log_info, log_error, log_info_simple, init_logging};
use crate::utils::{setup_directories};
use crate::processor::{FileProcessor, log_processing_result};
use crate::config::AppConfig;

mod error;
mod watcher;
mod processor;
mod logging;
mod utils;
mod config;

use watcher::watch_files;

/// starts a file watcher, and processes new files via an MPSC channel.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = AppConfig::load_or_default().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    
    // Initialize logging with configuration
    init_logging(&config.logging).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    
    log_info_simple("Starting Concurrent File Processor service...");
    
    let input_dir = config.input_dir();
    let output_dir = config.output_dir();

    setup_directories(&input_dir, &output_dir).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    
    // Create a Multi-Producer, Single-Consumer (MPSC) channel.
    let (tx, mut rx) = mpsc::channel::<PathBuf>(config.watcher.channel_buffer_size);

    // Create file processor with configuration
    let processor = FileProcessor::new(config.processing.clone());
    processor.process_initial_files(&input_dir, &output_dir).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    let watcher_input_dir = input_dir.clone();
    let watcher_tx = tx.clone(); // Clone the sender for the watcher task.
    let watcher_config = config.watcher.clone();
    tokio::spawn(async move {
        log_info_simple("Starting file watcher...");
        if let Err(e) = watch_files(&watcher_input_dir, watcher_tx, &watcher_config).await {
            log_error("File watcher experienced an error", &e);
        }
    });

    // --- File Processor Task ---
    log_info_simple("Starting main file processing loop...");
    while let Some(file_path) = rx.recv().await {
        log_info("Received new file for processing", &format!("{}", file_path.display()));
        let output_dir_clone = output_dir.clone(); // Clone for each spawned task.
        let processor_clone = processor.clone(); // Clone processor for each task

        tokio::spawn(async move {
            match processor_clone.process_file(&file_path, &output_dir_clone).await {
                Ok(result) => {
                    log_processing_result("File processing", &result);
                }
                Err(e) => {
                    log_error("Failed to process new file", &format!("{}: {}", file_path.display(), e));
                }
            }
        });
    }

    log_info_simple("Concurrent File Processor service stopped gracefully.");
    Ok(())
}
