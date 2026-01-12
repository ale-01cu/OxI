mod db;
mod indexer;
mod types;

use db::Database;
use indexer::Indexer;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use dirs;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use tracing::{error, info};
use tracing_subscriber;
use types::{IndexingStatus, SearchConfig, SearchFilters, SearchResults};

static DB_PATH: &str = "oxi-search.db";

fn get_db_path() -> PathBuf {
    if cfg!(debug_assertions) {
        // En desarrollo, usar el directorio de datos del usuario
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("OxI Search");
        std::fs::create_dir_all(&path).unwrap_or_default();
        path.push("oxi-search.db");
        path
    } else {
        // En producción, usar el directorio actual
        PathBuf::from(DB_PATH)
    }
}

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
            |(path, name, extension, file_size, is_dir, modified_time)| types::SearchResult {
                path,
                name,
                extension,
                file_size: file_size.map(|s| s as u64),
                is_dir,
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

    let app = Arc::new(app_handle);

    tokio::spawn(async move {
        let app_clone = app.clone();
        let progress_callback = Arc::new(move |progress: types::IndexingProgress| {
            info!("Indexing progress: {:?}", progress);
            let _ = app_clone.emit("indexing-progress", progress);
        });

        let result = indexer
            .index_multiple_paths(paths_to_index, patterns, progress_callback)
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
async fn minimize_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.minimize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn toggle_maximize_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let is_maximized = window.is_maximized().map_err(|e| e.to_string())?;
        if is_maximized {
            window.unmaximize().map_err(|e| e.to_string())?;
        } else {
            window.maximize().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
async fn close_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn start_dragging(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.start_dragging().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn open_location(path: String) -> Result<(), String> {

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

    let db_path = get_db_path();
    let db = match Database::new(db_path) {
        Ok(db) => Arc::new(Mutex::new(db)),
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            panic!("Database initialization failed: {}", e);
        }
    };

    info!("Database initialized");

    let db_for_tauri = Arc::clone(&db);
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            let quit_i = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Mostrar OxI Search", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            let db_for_setup = Arc::clone(&db);
            let app_handle = app.handle().clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    let file_count = {
                        let db_guard = db_for_setup.lock().unwrap();
                        db_guard.get_file_count().unwrap_or(0)
                    };

                    if file_count == 0 {
                        info!("No files indexed yet, starting automatic indexing");
                        let indexer = Indexer::new(db_for_setup);

                        let paths_to_index = Indexer::get_default_indexing_paths();
                        let patterns = Indexer::get_default_exclude_patterns();

                        let app_clone = app_handle.clone();
                        let progress_callback = Arc::new(move |progress: types::IndexingProgress| {
                            info!("Auto-indexing progress: {:?}", progress);
                            let _ = app_clone.emit("indexing-progress", progress);
                        });

                        let result = indexer
                            .index_multiple_paths(paths_to_index, patterns, progress_callback)
                            .await;

                        match result {
                            Ok(count) => {
                                info!("Auto-indexing completed: {} files", count);
                                let _ = app_handle.emit("indexing-completed", count);
                            }
                            Err(e) => {
                                error!("Auto-indexing failed: {}", e);
                                let _ = app_handle.emit("indexing-error", e.to_string());
                            }
                        }
                    } else {
                        info!("Database already contains {} files, skipping auto-index", file_count);
                    }
                });
            });

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .manage(db_for_tauri)
        .invoke_handler(tauri::generate_handler![
            search_files,
            reindex_path,
            get_indexing_status,
            get_config,
            update_config,
            open_location,
            minimize_window,
            toggle_maximize_window,
            close_window,
            start_dragging,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
