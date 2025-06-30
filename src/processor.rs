use crate::error::AppError;
use crate::logging::{log_info, log_error, log_info_simple, log_error_simple};
use crate::config::ProcessingConfig;
use crate::utils::{
    wait_for_file_availability, read_file_content, extract_file_name,
    write_processed_data, remove_original_file, scan_input_directory,
    hash_file_content
};

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task;

/// Trait for defining file processing strategies
/// This allows for flexible implementation of different processing logic
pub trait ProcessingStrategy: Send + Sync {
    /// Process file content and return processed data as string
    fn process_content(&self, file_name: &str, content: &[u8]) -> Result<String, AppError>;
}

/// Metadata for file processing (general for any processing strategy)
#[derive(Debug, Clone)]
pub struct ProcessingMetadata {
    pub original_size: usize,
    pub processed_size: usize,
    pub processing_time_ms: u64,
    pub strategy_info: Option<String>,
}

/// Default processing strategy that hashes file content
#[derive(Clone)]
pub struct HashProcessingStrategy;

impl ProcessingStrategy for HashProcessingStrategy {
    fn process_content(&self, file_name: &str, content: &[u8]) -> Result<String, AppError> {
        use sha2::{Sha256, Digest};
        use hex;
        
        let hash = {
            let mut hasher = Sha256::new();
            hasher.update(content);
            let result = hasher.finalize();
            hex::encode(result)
        };
        
        let processed_content = format!(
            "Processed content for {}: Data size {}\nSHA256: {}",
            file_name,
            content.len(),
            hash
        );
        
        Ok(processed_content)
    }
}

/// Example custom processing strategy that counts words and characters
#[derive(Clone)]
pub struct TextAnalysisStrategy;

impl ProcessingStrategy for TextAnalysisStrategy {
    fn process_content(&self, file_name: &str, content: &[u8]) -> Result<String, AppError> {
        // Convert bytes to string (assuming UTF-8)
        let text = String::from_utf8_lossy(content);
        
        let word_count = text.split_whitespace().count();
        let char_count = text.chars().count();
        let line_count = text.lines().count();
        
        let processed_content = format!(
            "Text analysis for {}: Data size {}\nWords: {}\nCharacters: {}\nLines: {}\nHash: {}",
            file_name,
            content.len(),
            word_count,
            char_count,
            line_count,
            hash_file_content(content)
        );
        
        Ok(processed_content)
    }
}

/// Represents the result of file processing
#[derive(Debug)]
pub struct ProcessingResult {
    pub input_file: PathBuf,
    pub output_file: PathBuf,
    pub original_size: usize,
    pub processed_size: usize,
    pub processing_time_ms: u64,
    pub strategy_info: Option<String>,
}

/// Main file processor that handles file operations and processing
pub struct FileProcessor {
    config: ProcessingConfig,
    strategy: Arc<dyn ProcessingStrategy>,
}

impl FileProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        Self { 
            config,
            //strategy: Arc::new(HashProcessingStrategy),
            strategy: Arc::new(TextAnalysisStrategy),
        }
    }
    
    /// Create a file processor with a custom processing strategy
    pub fn with_strategy(config: ProcessingConfig, strategy: Arc<dyn ProcessingStrategy>) -> Self {
        Self { config, strategy }
    }

    /// Process a single file from input to output directory
    pub async fn process_file(&self, file_path: &Path, output_dir: &Path) -> Result<ProcessingResult, AppError> {
        log_info("Processing file", &format!("{}", file_path.display()));

        wait_for_file_availability(file_path, &self.config).await?;

        let file_content = read_file_content(file_path).await?;
        let original_size = file_content.len();
        
        let file_name = extract_file_name(file_path)?;
        
        // Measure processing time
        let start_time = std::time::Instant::now();
        let processed_data = self.process_content_in_background(&file_name, &file_content).await?;
        let processing_time = start_time.elapsed();
        
        let output_path = write_processed_data(&file_name, &processed_data, output_dir, &self.config.output_extension).await?;
        
        remove_original_file(file_path).await?;

        log_info("Successfully processed file", &format!("{}", file_path.display()));
        
        // Create general metadata
        let metadata = ProcessingMetadata {
            original_size,
            processed_size: processed_data.len(),
            processing_time_ms: processing_time.as_millis() as u64,
            strategy_info: None, // Strategies can set this if they want
        };
        
        Ok(ProcessingResult {
            input_file: file_path.to_path_buf(),
            output_file: output_path,
            original_size: metadata.original_size,
            processed_size: metadata.processed_size,
            processing_time_ms: metadata.processing_time_ms,
            strategy_info: metadata.strategy_info,
        })
    }

    /// Process all existing files in the input directory
    pub async fn process_initial_files(&self, input_dir: &Path, output_dir: &Path) -> Result<(), AppError> {
        log_info_simple("Processing existing files in input directory...");
        
        let existing_files = scan_input_directory(input_dir).await?;
        
        if !existing_files.is_empty() {
            log_info("Found existing files to process", &format!("{} files", existing_files.len()));
            self.spawn_processing_tasks(existing_files, output_dir).await;
        } else {
            log_info_simple("No existing files found in input directory.");
        }

        Ok(())
    }

    // Private helper methods

    /// Process content in background thread (CPU-intensive operations)
    async fn process_content_in_background(&self, file_name: &str, content: &[u8]) -> Result<String, AppError> {
        let content_clone = content.to_vec();
        let file_name_clone = file_name.to_string();
        let strategy_clone = Arc::clone(&self.strategy);
        
        task::spawn_blocking(move || {
            log_info("Starting CPU-bound processing", &format!("for '{}' on a blocking thread", file_name_clone));
            
            let result = strategy_clone.process_content(&file_name_clone, &content_clone);
            
            log_info("Finished CPU-bound processing", &format!("for '{}'", file_name_clone));
            result
        }).await.map_err(|e| {
            let msg = format!("Blocking task failed: {}", e);
            log_error_simple(&msg);
            AppError::Processing(msg)
        })?
    }

    async fn spawn_processing_tasks(&self, files: Vec<PathBuf>, output_dir: &Path) {
        for file_path in files {
            let output_dir_clone = output_dir.to_path_buf();
            let processor = self.clone();

            tokio::spawn(async move {
                log_info("Processing existing file", &format!("{}", file_path.display()));
                match processor.process_file(&file_path, &output_dir_clone).await {
                    Ok(result) => {
                        log_processing_result("Existing file processing", &result);
                    }
                    Err(e) => {
                        log_error("Failed to process existing file", &format!("{}: {}", file_path.display(), e));
                    }
                }
            });
        }
    }
}

// Implement Clone for FileProcessor to allow spawning in async tasks
impl Clone for FileProcessor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            strategy: Arc::clone(&self.strategy),
        }
    }
}

/// Helper function to log processing results consistently
pub fn log_processing_result(context: &str, result: &ProcessingResult) {
    let strategy_info = result.strategy_info.as_deref().unwrap_or("None");
    log_info(&format!("{} completed successfully", context), &format!(
        "Input: {}, Output: {}, Original: {} bytes, Processed: {} bytes, Time: {}ms, Strategy Info: {}",
        result.input_file.display(),
        result.output_file.display(),
        result.original_size,
        result.processed_size,
        result.processing_time_ms,
        strategy_info
    ));
}