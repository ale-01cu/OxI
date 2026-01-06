mod db;
mod indexer;
mod types;

use db::Database;
use indexer::Indexer;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::Mutex as AsyncMutex;
use tracing::{error, info};
use tracing_subscriber;
use types::{IndexingProgress, IndexingStatus, SearchConfig, SearchFilters, SearchResults};

static DB_PATH: &str = "oxi-search.db";

#[tauri::command]
async fn search_files(
    query: String,
    filters: SearchFilters,
    page: usize,
    limit: usize,
    db: tauri::State<'_, Arc<Mutex<Database>>>,
) -> Result<SearchResults, String> {
    if query.is_empty() {
        return Ok(SearchResults {
            query,
            results: Vec::new(),
            total: 0,
            page,
            limit,
        });
    }

    let db_guard = db.lock().map_err(|e| e.to_string())?;
    let results = db_guard
        .search_files(
            &query,
            filters.extensions,
            filters.min_size.map(|s| s as i64),
            filters.max_size.map(|s| s as i64),
            limit,
        )
        .map_err(|e| e.to_string())?;

    let total = results.len();

    let results: Vec<types::SearchResult> = results
        .into_iter()
        .map(
            |(path, name, extension, file_size, modified_time)| types::SearchResult {
                path,
                name,
                extension,
                file_size: file_size.map(|s| s as u64),
                modified_time,
                score: 1.0,
            },
        )
        .collect();

    Ok(SearchResults {
        query,
        results,
        total,
        page,
        limit,
    })
}

#[tauri::command]
async fn reindex_path(
    path: Option<String>,
    exclude_patterns: Vec<String>,
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let db_clone = Arc::clone(&db);
    let indexer = Indexer::new(db_clone);

    let paths_to_index = if let Some(p) = path {
        vec![p]
    } else {
        Indexer::get_default_indexing_paths()
    };

    let patterns = if exclude_patterns.is_empty() {
        Indexer::get_default_exclude_patterns()
    } else {
        exclude_patterns
    };

    info!("Starting reindex of {:?} paths", paths_to_index);

    let app = app_handle.clone();

    tokio::spawn(async move {
        let result = indexer
            .index_multiple_paths(&paths_to_index, &patterns, |progress| {
                info!("Indexing progress: {:?}", progress);
                let _ = app.emit("indexing-progress", progress);
            })
            .await;

        match result {
            Ok(count) => {
                info!("Indexing completed: {} files", count);
                let _ = app.emit("indexing-completed", count);
            }
            Err(e) => {
                error!("Indexing failed: {}", e);
                let _ = app.emit("indexing-error", e.to_string());
            }
        }
    });

    Ok("Indexing started".to_string())
}

#[tauri::command]
async fn get_indexing_status(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
) -> Result<IndexingStatus, String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;
    let file_count = db_guard.get_file_count().map_err(|e| e.to_string())?;
    let database_size = db_guard.get_database_size().map_err(|e| e.to_string())?;
    let last_indexed = db_guard
        .get_last_indexed_time()
        .map_err(|e| e.to_string())?;

    Ok(IndexingStatus {
        is_indexing: false,
        last_indexed,
        total_files: file_count,
        database_size,
    })
}

#[tauri::command]
async fn get_config() -> Result<SearchConfig, String> {
    Ok(SearchConfig::default())
}

#[tauri::command]
async fn update_config(config: SearchConfig) -> Result<(), String> {
    info!("Config updated: {:?}", config);
    Ok(())
}

#[tauri::command]
async fn open_location(path: String) -> Result<(), String> {
    use tauri_plugin_shell::ShellExt;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        if std::path::Path::new(&path).is_dir() {
            std::process::Command::new("xdg-open")
                .arg(&path)
                .spawn()
                .map_err(|e| e.to_string())?;
        } else {
            let parent = std::path::Path::new(&path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());

            std::process::Command::new("xdg-open")
                .arg(&parent)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("OxI Search starting...");

    let db_path = PathBuf::from(DB_PATH);
    let db = match Database::new(db_path) {
        Ok(db) => Arc::new(Mutex::new(db)),
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            panic!("Database initialization failed: {}", e);
        }
    };

    info!("Database initialized");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            search_files,
            reindex_path,
            get_indexing_status,
            get_config,
            update_config,
            open_location,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
