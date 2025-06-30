//! Example demonstrating custom processing strategies
//! 
//! This example shows how to implement custom file processing logic
//! using the ProcessingStrategy trait.

use concurrent_file_processor::processor::{ProcessingStrategy, FileProcessor};
use concurrent_file_processor::config::ProcessingConfig;
use concurrent_file_processor::error::AppError;
use std::sync::Arc;

/// Example: Image metadata extraction strategy
pub struct ImageMetadataStrategy;

impl ProcessingStrategy for ImageMetadataStrategy {
    fn process_content(&self, file_name: &str, content: &[u8]) -> Result<String, AppError> {
        // Simulate image metadata extraction
        let file_size = content.len();
        let image_type = if file_name.ends_with(".jpg") || file_name.ends_with(".jpeg") {
            "JPEG"
        } else if file_name.ends_with(".png") {
            "PNG"
        } else if file_name.ends_with(".gif") {
            "GIF"
        } else {
            "Unknown"
        };
        
        // Simulate extracting dimensions (in real implementation, you'd use image libraries)
        let width = 1920;
        let height = 1080;
        
        let processed_content = format!(
            "Image analysis for {}: Data size {}\nType: {}\nDimensions: {}x{}\nHash: {}",
            file_name,
            file_size,
            image_type,
            width,
            height,
            concurrent_file_processor::utils::hash_file_content(content)
        );
        
        Ok(processed_content)
    }
}

/// Example: CSV validation strategy
pub struct CsvValidationStrategy;

impl ProcessingStrategy for CsvValidationStrategy {
    fn process_content(&self, file_name: &str, content: &[u8]) -> Result<String, AppError> {
        let text = String::from_utf8_lossy(content);
        let lines: Vec<&str> = text.lines().collect();
        
        if lines.is_empty() {
            return Err(AppError::Processing("CSV file is empty".to_string()));
        }
        
        let header = lines[0];
        let columns = header.split(',').count();
        let data_rows = lines.len() - 1;
        
        // Validate that all rows have the same number of columns
        let mut validation_errors = Vec::new();
        for (i, line) in lines.iter().enumerate().skip(1) {
            let row_columns = line.split(',').count();
            if row_columns != columns {
                validation_errors.push(format!("Row {} has {} columns, expected {}", i + 1, row_columns, columns));
            }
        }
        
        let is_valid = validation_errors.is_empty();
        let status = if is_valid { "VALID" } else { "INVALID" };
        
        let processed_content = format!(
            "CSV validation for {}: Data size {}\nStatus: {}\nColumns: {}\nRows: {}\nErrors: {}\nHash: {}",
            file_name,
            content.len(),
            status,
            columns,
            data_rows,
            if validation_errors.is_empty() { "None".to_string() } else { validation_errors.join("; ") },
            concurrent_file_processor::utils::hash_file_content(content)
        );
        
        Ok(processed_content)
    }
}

/// Example: Encryption strategy
pub struct EncryptionStrategy {
    key: String,
}

impl EncryptionStrategy {
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

impl ProcessingStrategy for EncryptionStrategy {
    fn process_content(&self, file_name: &str, content: &[u8]) -> Result<String, AppError> {
        // Simple XOR encryption (for demonstration - use proper encryption in production)
        let key_bytes = self.key.as_bytes();
        let mut encrypted = Vec::new();
        
        for (i, &byte) in content.iter().enumerate() {
            let key_byte = key_bytes[i % key_bytes.len()];
            encrypted.push(byte ^ key_byte);
        }
        
        let encrypted_hex = encrypted.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join("");
        
        let processed_content = format!(
            "Encrypted content for {}: Data size {}\nEncryption: XOR\nKey length: {}\nEncrypted data: {}\nHash: {}",
            file_name,
            content.len(),
            self.key.len(),
            encrypted_hex,
            concurrent_file_processor::utils::hash_file_content(content)
        );
        
        Ok(processed_content)
    }
}

/// Example usage function
pub fn demonstrate_custom_strategies() {
    let config = ProcessingConfig::default();
    
    // Create processors with different strategies
    let image_processor = FileProcessor::with_strategy(
        config.clone(),
        Arc::new(ImageMetadataStrategy)
    );
    
    let csv_processor = FileProcessor::with_strategy(
        config.clone(),
        Arc::new(CsvValidationStrategy)
    );
    
    let encryption_processor = FileProcessor::with_strategy(
        config,
        Arc::new(EncryptionStrategy::new("my-secret-key".to_string()))
    );
    
    println!("Created processors with custom strategies:");
    println!("- Image metadata processor");
    println!("- CSV validation processor");
    println!("- Encryption processor");
}