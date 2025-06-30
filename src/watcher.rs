use crate::error::AppError;
use crate::logging::{log_info, log_error, log_debug};
use crate::config::WatcherConfig;

use std::{
    path::{Path, PathBuf},
};

use tokio::{
    sync::mpsc,
    time::{sleep, Duration},
};

pub async fn watch_files(path: &Path, tx: mpsc::Sender<PathBuf>, config: &WatcherConfig) -> Result<(), AppError> {
    use notify::{recommended_watcher, EventKind, Watcher};

    let (tx_notify, mut rx_notify) = mpsc::channel(config.channel_buffer_size);
    let mut watcher = recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        if let Err(e) = tx_notify.blocking_send(res) {
            log_error("Failed to send notify event", &e);
        }
    }).map_err(|e| {
        log_error("Failed to create file watcher", &e);
        AppError::Watch(e)
    })?;

    let recursive_mode = if config.recursive {
        notify::RecursiveMode::Recursive
    } else {
        notify::RecursiveMode::NonRecursive
    };

    watcher.watch(path, recursive_mode).map_err(|e| {
        log_error("Failed to watch directory", &format!("{}: {}", path.display(), e));
        AppError::Watch(e)
    })?;
    
    log_info("Watching directory", &format!("{}", path.display()));

    while let Some(res) = rx_notify.recv().await {
        match res {
            Ok(event) => {
                log_debug("Received watch event", &format!("{:?}", event));
                if let EventKind::Create(..) = event.kind {
                    for path in event.paths {
                        if path.is_file() {
                            log_info("New file detected", &format!("{}", path.display()));
                            
                            // Add a small delay to allow file system operations to complete
                            sleep(Duration::from_millis(config.processing_delay_ms)).await;
                            
                            // Send the detected file path to the processing channel.
                            if let Err(e) = tx.send(path).await {
                                log_error("Failed to send file path to processor", &e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log_error("Watch error", &e);
                return Err(AppError::Watch(e));
            }
        }
    }
    Ok(())
}