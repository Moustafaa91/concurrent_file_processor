# Concurrent File Processor

A Rust application that watches for new files in an input directory, processes them concurrently using flexible processing strategies, and saves the results to an output directory.

## Prerequisites

- **Rust**: Version 1.70 or higher
- **Operating System**: Windows, macOS, or Linux
- **File System**: Read/write permissions for input and output directories

## Features

- **File Watching**: Monitors input directory for new files using `notify`
- **Concurrent Processing**: Processes multiple files simultaneously using Tokio
- **Flexible Processing**: Plugin-based architecture for custom file processing strategies
- **Configurable**: All settings are configurable via TOML file
- **Logging**: Comprehensive logging with configurable levels
- **Error Handling**: Robust error handling with retry mechanisms
- **File Lock Detection**: Handles files that are temporarily locked by other processes

## Quick Start

1. **Clone and Build**:
   ```bash
   git clone https://github.com/Moustafaa91/concurrent_file_processor
   cd concurrent_file_processor
   cargo build 
   ```

2. **Run the Application**: 
   ```bash
   # Make sure to add an environment variable named RUST_LOG and value could be (info, error, warn, debug) 
   cargo run
   ```
   
   ```bash
   # or, use this command for Windows OS
   $env:RUST_LOG="info"; cargo run
   ```
   
   ```bash
   # Linux
   RUST_LOG=info cargo run
   ```

4. **Test with Files**:
   ```bash
   # Add some test files to the input directory
   echo "Hello, World!" > input_files/test.txt
   ```

## Flexible Processing System

The application uses a **Strategy Pattern** to allow for flexible file processing. This means you can implement any type of file processing logic without modifying the core application code.

### Built-in Processing Strategies

1. **HashProcessingStrategy**: Computes SHA256 hash of file content
2. **TextAnalysisStrategy**: Analyzes text files for word count, character count, and line count

### Creating Custom Processing Strategies

You can implement your own processing logic by implementing the `ProcessingStrategy` trait. **Only one function is required**:

```rust
use crate::processor::ProcessingStrategy;
use crate::error::AppError;

pub struct MyCustomStrategy;  // No Clone derive needed

impl ProcessingStrategy for MyCustomStrategy {
    fn process_content(&self, file_name: &str, content: &[u8]) -> Result<String, AppError> {
        // Your custom processing logic here
        // For example: image compression, text translation, data validation, etc.
        
        let processed_data = format!(
            "Custom processing for {}: Data size {}\nCustom result: {}\nHash: {}",
            file_name,
            content.len(),
            "your processing result",
            crate::utils::hash_file_content(content)  // Include hash for metadata
        );
        
        Ok(processed_data)
    }
}
```

### Using Custom Strategies

```rust
use crate::processor::{FileProcessor, MyCustomStrategy, ProcessingConfig};
use std::sync::Arc;

// Create processor with custom strategy
let config = ProcessingConfig::default();
let custom_strategy = Arc::new(MyCustomStrategy);
let processor = FileProcessor::with_strategy(config, custom_strategy);

// Use the processor in your application
processor.process_file(&file_path, &output_dir).await?;
```

### Example Use Cases

- **Image Processing**: Resize, compress, or convert image formats
- **Text Analysis**: Sentiment analysis, keyword extraction, language detection
- **Data Validation**: Validate CSV, JSON, or XML files
- **File Conversion**: Convert between different file formats
- **Content Filtering**: Remove sensitive information or apply content filters
- **Encryption/Decryption**: Encrypt or decrypt file contents
- **Compression**: Compress or decompress files

## Configuration

The application uses a `config.toml` file for all configuration settings. If no configuration file exists, a default one will be created automatically.

### Configuration File Structure

```toml
[processing]
# Maximum number of retries for file operations
max_retries = 10
# Initial retry delay in milliseconds
initial_retry_delay_ms = 100
# Maximum retry delay in seconds
max_retry_delay_sec = 2
# Output file extension for processed files
output_extension = ".processed.txt"
# File locked error code for Windows systems
file_locked_error_code = 32

[directories]
# Input directory where files to be processed are placed
input_dir = "./input_files"
# Output directory where processed files are saved
output_dir = "./output_files"

[logging]
# Log level: debug, info, warn, error
level = "info"
# Directory where log files are stored
log_dir = "logs"
# Base name for log files (will be appended with timestamp)
log_basename = "app_log"
# Whether to duplicate logs to stdout
duplicate_to_stdout = true

[watcher]
# Channel buffer size for watcher events
channel_buffer_size = 32
# Delay in milliseconds before processing new files
processing_delay_ms = 50
# Whether to watch directories recursively
recursive = true
```

### Configuration Options

#### Processing Configuration
- `max_retries`: Number of times to retry file operations when files are locked
- `initial_retry_delay_ms`: Initial delay between retries in milliseconds
- `max_retry_delay_sec`: Maximum delay between retries in seconds
- `output_extension`: Extension for processed output files
- `file_locked_error_code`: Windows-specific error code for locked files

#### Directory Configuration
- `input_dir`: Path to the directory containing files to be processed
- `output_dir`: Path to the directory where processed files will be saved

#### Logging Configuration
- `level`: Log level (debug, info, warn, error)
- `log_dir`: Directory where log files are stored
- `log_basename`: Base name for log files
- `duplicate_to_stdout`: Whether to also output logs to console

#### Watcher Configuration
- `channel_buffer_size`: Buffer size for file watcher events
- `processing_delay_ms`: Delay before processing newly detected files
- `recursive`: Whether to watch subdirectories recursively

## Usage

1. **Install Dependencies**:
   ```bash
   cargo build
   ```

2. **Run the Application**:
   ```bash
   cargo run
   ```

3. **Add Files for Processing**:
   Place files in the `input_files` directory (or the directory specified in your config)

4. **Check Results**:
   Processed files will appear in the `output_files` directory

5. **Monitor Logs**:
   Check the `logs` directory for detailed processing information

## Developmet

### Building for Production
```bash
cargo build --release
```

### Logging During Development
change *RUST_LOG=debug* before *cargo run*

## Troubleshooting

### Common Issues

1. **Permission Denied**:
   - Ensure the application has read/write permissions for input/output directories
   - On Windows, run as administrator if needed

2. **Files Not Being Processed**:
   - Check that files are placed in the correct input directory
   - Verify the file watcher is working by checking logs
   - Ensure files are not locked by other applications

3. **High CPU Usage**:
   - Adjust `processing_delay_ms` in config to reduce processing frequency
   - Consider using a more efficient processing strategy

4. **Memory Issues**:
   - Large files are loaded entirely into memory
   - Consider implementing streaming processing for very large files

## Production Deployment

For production deployment, consider the following:

1. **Configuration Management**: Use environment-specific config files or environment variables
2. **Logging**: Configure appropriate log levels and rotation
3. **File Permissions**: Ensure proper read/write permissions for input/output directories
4. **Monitoring**: Set up monitoring for the application logs and file processing metrics
5. **Processing Strategy Selection**: Choose appropriate processing strategies based on your use case
6. **Resource Limits**: Monitor CPU and memory usage, especially for large files
7. **Backup Strategy**: Implement backup procedures for processed files

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement your changes
4. Add tests for new functionality if needed
5. Submit a pull request
