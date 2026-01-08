use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub file_size: Option<u64>,
    pub modified_time: DateTime<Utc>,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub file_size: Option<u64>,
    pub is_dir: bool,
    pub modified_time: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub extensions: Option<Vec<String>>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub min_date: Option<String>,
    pub max_date: Option<String>,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            extensions: None,
            min_size: None,
            max_size: None,
            min_date: None,
            max_date: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total: usize,
    pub page: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingProgress {
    pub current_path: String,
    pub files_processed: usize,
    pub total_files: Option<usize>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingStatus {
    pub is_indexing: bool,
    pub last_indexed: Option<String>,
    pub total_files: usize,
    pub database_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub indexing_paths: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_results: usize,
    pub fuzzy_threshold: f64,
    pub cache_enabled: bool,
    pub cache_ttl_hours: u64,
    pub theme: String,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            indexing_paths: vec![],
            exclude_patterns: vec![],
            max_results: 1000,
            fuzzy_threshold: 0.7,
            cache_enabled: true,
            cache_ttl_hours: 1,
            theme: "dark".to_string(),
        }
    }
}
