use crate::db::Database;
use crate::mft_indexer::MftIndexer;
use crate::types::{FileRecord, IndexingProgress};
use chrono::{DateTime, Utc};
use ignore::WalkBuilder;
use std::path::{Path};
use std::sync::Arc;
use std::time::Instant;
use std::sync::Mutex;
use tracing::{info, warn};

pub struct Indexer {
    db: Arc<Mutex<Database>>,
}

impl Indexer {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }

    fn is_windows_drive(path: &str) -> bool {
        #[cfg(windows)]
        {
            let path_upper = path.to_uppercase();
            path_upper.len() == 3 && path_upper.chars().nth(1) == Some(':')
                && path_upper.chars().nth(2) == Some('\\')
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    fn can_use_mft(path: &str) -> bool {
        if !Self::is_windows_drive(path) {
            return false;
        }

        #[cfg(windows)]
        {
            let drive = path.chars().next().unwrap();
            let drive_path = format!(r"\\.\{}:", drive);
            std::fs::File::open(&drive_path).is_ok()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    pub async fn index_path(
        &self,
        path: &str,
        exclude_patterns: Vec<String>,
        progress_callback: Arc<dyn Fn(IndexingProgress) + Send + Sync>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        info!("Starting indexing of path: {}", path);

        if Self::is_windows_drive(path) && Self::can_use_mft(path) {
            info!("Attempting MFT indexing for drive: {}", path);
            let drive = path.chars().next().unwrap();
            let mft_indexer = MftIndexer::new(Arc::clone(&self.db));
            match mft_indexer
                .index_drive(&drive.to_string(), progress_callback.clone())
                .await
            {
                Ok(count) => {
                    info!("MFT indexing successful: {} files", count);
                    return Ok(count);
                }
                Err(e) => {
                    warn!("MFT indexing failed: {}. Falling back to filesystem walk.", e);
                }
            }
        }

        info!("Using filesystem walk for path: {}", path);
        let start = Instant::now();

        let path_obj = Path::new(path);

        if !path_obj.exists() {
            return Err(format!("Path does not exist: {}", path).into());
        }

        let mut walk = WalkBuilder::new(path_obj);
        walk.hidden(true);

        for pattern in &exclude_patterns {
            let pattern = pattern.clone();
            walk.filter_entry(move |entry| {
                let path_str = entry.path().to_string_lossy();
                !path_str.contains(&pattern)
            });
        }

        let walker = walk.build();

        const BATCH_SIZE: usize = 5_000;
        let mut batch_buffer: Vec<FileRecord> = Vec::with_capacity(BATCH_SIZE);

        // "Procesados" (para progreso) vs "persistidos" (para retorno).
        let mut processed = 0usize;
        let mut persisted = 0usize;

        let flush_batch = |batch: &mut Vec<FileRecord>| -> Result<usize, Box<dyn std::error::Error>> {
            if batch.is_empty() {
                return Ok(0);
            }

            let mut db_guard = self
                .db
                .lock()
                .map_err(|e| format!("Failed to lock database: {}", e))?;

            let batch_len = batch.len();

            match db_guard.upsert_batch(batch.as_slice()) {
                Ok(()) => {
                    batch.clear();
                    Ok(batch_len)
                }
                Err(e) => {
                    warn!("Batch upsert falló ({} items): {}. Haciendo fallback item-por-item.", batch_len, e);

                    let mut ok_count = 0usize;
                    for r in batch.iter() {
                        if let Err(item_err) = db_guard.upsert_file(
                            r.path.as_str(),
                            r.name.as_str(),
                            r.extension.as_deref(),
                            r.file_size,
                            r.is_dir,
                            r.modified_time.as_str(),
                            r.last_indexed.as_str(),
                        ) {
                            warn!("Failed to upsert {}: {}", r.path, item_err);
                        } else {
                            ok_count += 1;
                        }
                    }

                    batch.clear();
                    Ok(ok_count)
                }
            }
        };

        for result in walker {
            if let Ok(entry) = result {
                if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                    if let Some(path_str) = entry.path().to_str() {
                        if let Some(name) = entry.file_name().to_str() {
                            let modified_time: DateTime<Utc> = Utc::now();
                            let modified_time_str = modified_time.to_rfc3339();
                            let last_indexed_str = Utc::now().to_rfc3339();

                            batch_buffer.push(FileRecord {
                                path: path_str.to_string(),
                                name: name.to_string(),
                                extension: None,
                                file_size: None,
                                is_dir: true,
                                modified_time: modified_time_str,
                                last_indexed: last_indexed_str,
                            });

                            processed += 1;
                            progress_callback(IndexingProgress {
                                current_path: path_str.to_string(),
                                files_processed: processed,
                                total_files: None,
                                status: "indexing".to_string(),
                            });

                            if batch_buffer.len() >= BATCH_SIZE {
                                persisted += flush_batch(&mut batch_buffer)?;
                            }
                        }
                    }
                } else if entry.file_type().map_or(false, |ft| ft.is_file()) {
                    if let Ok(metadata) = entry.metadata() {
                        if let Some(path_str) = entry.path().to_str() {
                            if let Some(name) = entry.file_name().to_str() {
                                let extension = entry
                                    .path()
                                    .extension()
                                    .and_then(|e| e.to_str())
                                    .map(|s| format!(".{}", s));

                                let modified_time: DateTime<Utc> = metadata
                                    .modified()
                                    .ok()
                                    .map(|t| DateTime::<Utc>::from(t))
                                    .unwrap_or_else(Utc::now);

                                let file_size = Some(metadata.len() as i64);
                                let modified_time_str = modified_time.to_rfc3339();
                                let last_indexed_str = Utc::now().to_rfc3339();

                                batch_buffer.push(FileRecord {
                                    path: path_str.to_string(),
                                    name: name.to_string(),
                                    extension,
                                    file_size,
                                    is_dir: false,
                                    modified_time: modified_time_str,
                                    last_indexed: last_indexed_str,
                                });

                                processed += 1;
                                progress_callback(IndexingProgress {
                                    current_path: path_str.to_string(),
                                    files_processed: processed,
                                    total_files: None,
                                    status: "indexing".to_string(),
                                });

                                if batch_buffer.len() >= BATCH_SIZE {
                                    persisted += flush_batch(&mut batch_buffer)?;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Guardar el remanente final.
        persisted += flush_batch(&mut batch_buffer)?;

        let elapsed = start.elapsed();
        info!(
            "Indexing completed: processed={} persisted={} in {:?}",
            processed,
            persisted,
            elapsed
        );

        Ok(persisted)
    }

    pub async fn index_multiple_paths(
        &self,
        paths: Vec<String>,
        exclude_patterns: Vec<String>,
        progress_callback: Arc<dyn Fn(IndexingProgress) + Send + Sync>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut total_count = 0;

        for (idx, path) in paths.iter().enumerate() {
            info!("Indexing path {}/{}: {}", idx + 1, paths.len(), path);
            let count = self
                .index_path(path, exclude_patterns.clone(), progress_callback.clone())
                .await?;
            total_count += count;
        }

        Ok(total_count)
    }

    pub fn get_default_indexing_paths() -> Vec<String> {
        let mut paths = Vec::new();

        #[cfg(unix)]
        {
            if let Ok(home) = std::env::var("HOME") {
                paths.push(home.clone());
            }

            let mount_file = "/proc/mounts";
            if let Ok(content) = std::fs::read_to_string(mount_file) {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let mount_point = parts[1];
                        let path = Path::new(mount_point);

                        if path.exists() && path.is_dir() {
                            let mount_path = mount_point.to_string();

                            let should_include = mount_path != "/" &&
                                               !mount_path.starts_with("/boot") &&
                                               !mount_path.starts_with("/dev") &&
                                               !mount_path.starts_with("/proc") &&
                                               !mount_path.starts_with("/sys") &&
                                               !mount_path.starts_with("/run") &&
                                               !mount_path.starts_with("/tmp") &&
                                               !mount_path.contains("/snap") &&
                                               !mount_path.starts_with("/var/lib");

                            if should_include && !paths.contains(&mount_path) {
                                paths.push(mount_path);
                            }
                        }
                    }
                }
            }
        }

        #[cfg(windows)]
        {
            for drive in b'A'..=b'Z' {
                let drive_path = format!("{}:\\", drive as char);
                if Path::new(&drive_path).exists() {
                    paths.push(drive_path);
                }
            }

            if let Ok(home) = std::env::var("USERPROFILE") {
                if !paths.contains(&home) {
                    paths.push(home);
                }
            }
        }

        if paths.is_empty() {
            if let Ok(home) = std::env::var("HOME") {
                paths.push(home);
            } else if let Ok(home) = std::env::var("USERPROFILE") {
                paths.push(home);
            }
        }

        paths
    }

    pub fn get_default_exclude_patterns() -> Vec<String> {
        vec![
            ".git".to_string(),
            "node_modules".to_string(),
            "target".to_string(),
            ".DS_Store".to_string(),
            "__pycache__".to_string(),
            ".venv".to_string(),
            "venv".to_string(),
        ]
    }
}
