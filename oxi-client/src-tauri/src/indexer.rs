use crate::db::Database;
use crate::types::{FileEntry, IndexingProgress};
use chrono::Utc;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

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
        exclude_patterns: &[String],
        progress_callback: impl Fn(IndexingProgress),
    ) -> Result<usize, Box<dyn std::error::Error>> {
        info!("Starting indexing of path: {}", path);
        let start = Instant::now();

        let path_obj = Path::new(path);

        if !path_obj.exists() {
            return Err(format!("Path does not exist: {}", path).into());
        }

        let mut walk = WalkBuilder::new(path_obj);
        walk.hidden(true);

        for pattern in exclude_patterns {
            walk.filter_entry(|entry| {
                let path_str = entry.path().to_string_lossy();
                !path_str.contains(pattern)
            });
        }

        let walker = walk.build_parallel();
        let count = Arc::new(Mutex::new(0usize));

        walker.run(|| {
            let db = Arc::clone(&self.db);
            let count = Arc::clone(&count);

            Box::new(move |result| {
                if let Ok(entry) = result {
                    if entry.file_type().is_file() {
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
                                        .and_then(|t| t.into())
                                        .unwrap_or_else(Utc::now);

                                    let file_size = Some(metadata.len() as i64);
                                    let modified_time_str = modified_time.to_rfc3339();
                                    let last_indexed_str = Utc::now().to_rfc3339();

                                    let db_clone = Arc::clone(&db);

                                    if let Ok(db) = db_clone.try_lock() {
                                        if let Err(e) = db.upsert_file(
                                            path_str,
                                            name,
                                            extension.as_deref(),
                                            file_size,
                                            &modified_time_str,
                                            &last_indexed_str,
                                        ) {
                                            warn!("Failed to upsert file {}: {}", path_str, e);
                                        } else {
                                            let mut cnt = count.lock().unwrap();
                                            *cnt += 1;

                                            progress_callback(IndexingProgress {
                                                current_path: path_str.to_string(),
                                                files_processed: *cnt,
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
                ignore::WalkState::Continue
            })
        });

        let elapsed = start.elapsed();
        let final_count = *count.lock().unwrap();
        info!("Indexing completed: {} files in {:?}", final_count, elapsed);

        Ok(final_count)
    }

    pub async fn index_multiple_paths(
        &self,
        paths: &[String],
        exclude_patterns: &[String],
        progress_callback: impl Fn(IndexingProgress),
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut total_count = 0;

        for (idx, path) in paths.iter().enumerate() {
            info!("Indexing path {}/{}: {}", idx + 1, paths.len(), path);
            let count = self
                .index_path(path, exclude_patterns, &progress_callback)
                .await?;
            total_count += count;
        }

        Ok(total_count)
    }

    pub fn get_default_indexing_paths() -> Vec<String> {
        let mut paths = Vec::new();

        if let Ok(home) = std::env::var("HOME") {
            paths.push(home.clone());
            paths.push(format!("{}/Documents", home));
            paths.push(format!("{}/Downloads", home));
            paths.push(format!("{}/Pictures", home));
        } else if let Ok(home) = std::env::var("USERPROFILE") {
            paths.push(home.clone());
            paths.push(format!("{}\\Documents", home));
            paths.push(format!("{}\\Downloads", home));
            paths.push(format!("{}\\Pictures", home));
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
