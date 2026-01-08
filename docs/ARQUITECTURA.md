# Arquitectura - OxI Search

## 1. Arquitectura General

```
┌─────────────────────────────────────────────────────────────────┐
│                    Interfaz de Usuario (React)                    │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                    Search Page                               │  │
│  │  ┌──────────────┐  ┌───────────────────────────────────┐  │  │
│  │  │ Command      │  │  Results List                    │  │  │
│  │  │ Palette      │  │  - File name + icon               │  │  │
│  │  │ (Ctrl+K)     │  │  - Path                           │  │  │
│  │  │              │  │  - Size, Date                     │  │  │
│  │  │ Search Input │  │  - Context menu                   │  │  │
│  │  │ Filters      │  │  - "Open location"                │  │  │
│  │  └──────────────┘  └───────────────────────────────────┘  │  │
│  │                                                            │  │
│  │  ┌────────────────────────────────────────────────────┐  │  │
│  │  │              Settings Page                          │  │  │
│  │  │  - Paths to index                                   │  │  │
│  │  │  - Paths to exclude                                 │  │  │
│  │  │  - Indexing schedule                                 │  │  │
│  │  │  - Theme (light/dark)                               │  │  │
│  │  └────────────────────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
        ┌────────────────────────────────────────────┐
        │         Tauri Bridge Layer                  │
        │  - Commands: search_files, reindex_path     │
        │  - Events: indexing_progress                │
        └────────────────────────────────────────────┘
                              │
                              ▼
    ┌───────────────────────────────────────────────────────┐
    │                Core Rust                              │
    │  ┌────────────────┐  ┌──────────────────────────┐    │
     │  │  Indexer       │  │   Searcher               │    │
     │  │  - walkdir     │  │   - Exact search         │    │
     │  │  - sequential  │  │   - Fuzzy search         │    │
     │  │  - auto-disk   │  │   - Regex support        │    │
     │  │  - auto-start  │  │                          │    │
     │  └────────────────┘  └──────────────────────────┘    │
    │  ┌────────────────┐  ┌──────────────────────────┐    │
    │  │  Cache         │  │   Filters                │    │
    │  │  - TTL         │  │   - By extension         │    │
    │  │  - invalidation│  │   - By size              │    │
    │  │                │  │   - By date               │    │
    │  └────────────────┘  └──────────────────────────┘    │
    └───────────────────────────────────────────────────────┘
                              │
                              ▼
        ┌────────────────────────────────────────────┐
        │            SQLite (local)                   │
        │  ┌──────────────────────────────────────┐  │
        │  │  search_index                        │  │
        │  │  - path (UNIQUE)                     │  │
        │  │  - name                              │  │
        │  │  - extension                         │  │
        │  │  - file_size                         │  │
        │  │  - modified_time                     │  │
        │  │  - last_indexed                      │  │
        │  └──────────────────────────────────────┘  │
        └────────────────────────────────────────────┘
```

## 2. Componentes Principales

### 2.1 Indexer (Rust)

**Responsabilidades**:
- Indexado recursivo del filesystem
- **Auto-indexing al iniciar la aplicación** (si DB está vacía)
- **Detección automática de discos** (Linux: /proc/mounts, Windows: letras de unidad)
- Indexación secuencial completa (evita conflictos con SQLite)
- Soporte para exclusiones (.gitignore-like)
- Progreso en tiempo real vía Tauri events

**Módulos**:
- `indexer.rs`: Core del indexador con walkdir
- `get_default_indexing_paths()`: Detección automática de discos
- `index_path()`: Indexación secuencial de paths
- `index_multiple_paths()`: Indexación de múltiples paths
- `get_default_exclude_patterns()`: Patterns de exclusión por defecto

**Flujo de indexación**:
1. **Auto-indexing en startup**:
   - Verificar si DB tiene archivos indexados
   - Si está vacía, iniciar auto-indexado automático
2. Detectar paths a indexar:
   - Linux: Leer /proc/mounts, excluir directorios del sistema
   - Windows: Detectar todas las letras de unidad (C:, D:, etc.)
3. Para cada path:
   - Recorrer con walkdir (secuencial para evitar race conditions)
   - Para cada archivo:
     - Extraer metadata (nombre, tamaño, fecha, extensión)
     - Upsert en SQLite
   - Emitir eventos de progreso en tiempo real
4. Completar y notificar a la UI

### 2.2 Searcher (Rust)

**Responsabilidades**:
- Búsqueda de archivos por query
- Soporte para búsqueda exacta y fuzzy
- Filtrado avanzado (tipo, tamaño, fecha)
- Ranking de resultados

**Módulos**:
- `searcher.rs`: Core del buscador
- `exact.rs`: Búsqueda exacta
- `fuzzy.rs`: Búsqueda fuzzy con Levenshtein
- `regex.rs`: Soporte regex
- `ranking.rs`: Algoritmo de ranking

**Flujo de búsqueda**:
1. Recibir query y filtros desde React
2. Validar query
3. Buscar en SQLite con FTS5 o LIKE
4. Aplicar filtros (WHERE clauses)
5. Ordenar por relevancia (ranking score)
6. Paginar resultados (LIMIT/OFFSET)
7. Devolver a UI

### 2.3 Cache (Rust)

**Responsabilidades**:
- Caché de búsquedas frecuentes
- TTL para invalidación
- Prefetch de resultados comunes

**Módulos**:
- `cache.rs`: Cache manager
- `invalidation.rs`: Lógica de invalidación

**Estrategia**:
- In-memory cache con LRU eviction
- TTL de 1 hora por defecto
- Invalidación cuando se reindexa un path

### 2.4 Tauri Commands

```rust
#[tauri::command]
async fn search_files(
    query: String,
    filters: SearchFilters,
    page: usize,
    limit: usize,
) -> Result<SearchResults, Error>

#[tauri::command]
async fn reindex_path(path: Option<String>, exclude_patterns: Vec<String>) -> Result<String, Error>

#[tauri::command]
async fn get_indexing_status() -> Result<IndexingStatus, Error>

#[tauri::command]
async fn get_config() -> Result<SearchConfig, Error>

#[tauri::command]
async fn update_config(config: SearchConfig) -> Result<(), Error>

#[tauri::command]
async fn open_location(path: String) -> Result<(), Error>
```

**Auto-indexing en Startup**:
```rust
.setup(move |app| {
    let db_for_setup = Arc::clone(&db);
    let app_handle = app.handle().clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let file_count = db_for_setup.lock().unwrap().get_file_count().unwrap_or(0);

            if file_count == 0 {
                info!("No files indexed yet, starting automatic indexing");
                // Iniciar auto-indexado automático
            }
        });
    });

    Ok(())
})
```

**Events**:
```rust
emit!("indexing-progress", progress);
emit!("indexing-completed", count);
emit!("indexing-error", error_message);
```

## 3. Base de Datos

### 3.1 Tabla: search_index

| Columna | Tipo | Descripción | Restricciones |
|---------|------|-------------|---------------|
| id | INTEGER | ID autoincremental | PRIMARY KEY |
| path | TEXT | Ruta completa | UNIQUE, NOT NULL |
| name | TEXT | Nombre del archivo | NOT NULL |
| extension | TEXT | Extensión | NULL |
| file_size | INTEGER | Tamaño en bytes | NULL |
| modified_time | TEXT | Última modificación | NOT NULL (ISO 8601) |
| last_indexed | TEXT | Última indexación | NOT NULL (ISO 8601) |

**SQL**:
```sql
CREATE TABLE search_index (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    extension TEXT,
    file_size INTEGER,
    modified_time TEXT NOT NULL,
    last_indexed TEXT NOT NULL
);

CREATE INDEX idx_search_name ON search_index(name);
CREATE INDEX idx_search_extension ON search_index(extension);
CREATE INDEX idx_search_size ON search_index(file_size);
CREATE INDEX idx_search_modified ON search_index(modified_time);
```

**Queries comunes**:
```sql
-- Búsqueda por nombre
SELECT * FROM search_index
WHERE name LIKE '%query%'
ORDER BY name ASC
LIMIT 50;

-- Filtrado por extensión
SELECT * FROM search_index
WHERE extension = '.pdf'
ORDER BY modified_time DESC
LIMIT 50;

-- Filtrado por tamaño (> 10MB)
SELECT * FROM search_index
WHERE file_size > 10485760
ORDER BY file_size DESC
LIMIT 50;
```

## 4. Patrón de Diseño

- **Repository Pattern**: Abstracción de DB
- **Strategy Pattern**: Diferentes algoritmos de búsqueda (exact, fuzzy, regex)
- **Observer Pattern**: Eventos de indexación vía Tauri events
- **Factory Pattern**: Creación de indexers por tipo de path

## 5. Concurrencia

- Tokio runtime async en Rust
- **Sequential indexing** (para evitar race conditions con SQLite)
- Mutex para acceso seguro a la base de datos
- Auto-indexing en thread separado con `std::thread::spawn`
- React state management para estado UI

## 6. Configuración

**Archivo**: `~/.oxi-search/config.toml`

```toml
[indexing]
paths = ["/home/user", "/home/user/Documents"]
exclude_patterns = [".git", "node_modules", "target"]
schedule = "daily"  # "manual", "hourly", "daily"

[search]
max_results = 1000
fuzzy_threshold = 0.7  # 0-1

[cache]
enabled = true
ttl_hours = 1

[ui]
theme = "dark"  # "light", "dark", "auto"
```
