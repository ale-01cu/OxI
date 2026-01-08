use crate::db::Database;
use crate::types::{IndexingProgress};
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

    pub async fn index_path(
        &self,
        path: &str,
        exclude_patterns: Vec<String>,
        progress_callback: Arc<dyn Fn(IndexingProgress) + Send + Sync>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        info!("Starting indexing of path: {}", path);
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
        let mut count = 0;

        for result in walker {
            if let Ok(entry) = result {
                if entry.file_type().map_or(false, |ft| ft.is_file()) {
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

                                let db_guard = self.db.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
                                if let Err(e) = db_guard.upsert_file(
                                    path_str,
                                    name,
                                    extension.as_deref(),
                                    file_size,
                                    &modified_time_str,
                                    &last_indexed_str,
                                ) {
                                    warn!("Failed to upsert file {}: {}", path_str, e);
                                } else {
                                    count += 1;

                                    progress_callback(IndexingProgress {
                                        current_path: path_str.to_string(),
                                        files_processed: count,
                                        total_files: None,
                                        status: "indexing".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        let elapsed = start.elapsed();
        info!("Indexing completed: {} files in {:?}", count, elapsed);

        Ok(count)
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
