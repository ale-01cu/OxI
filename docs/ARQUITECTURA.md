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
    │  │  - parallel    │  │   - Fuzzy search         │    │
    │  │  - incremental │  │   - Regex support        │    │
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
- Indexación incremental (solo cambios)
- Detección de archivos eliminados
- Soporte para exclusiones (.gitignore-like)

**Módulos**:
- `indexer.rs`: Core del indexador con walkdir
- `incremental.rs`: Lógica para indexado incremental
- `exclusions.rs`: Manejo de patterns de exclusión
- `progress.rs`: Reporte de progreso

**Flujo de indexación**:
1. Leer paths configurados
2. Para cada path:
   - Recorrer con walkdir (paralelo si es grande)
   - Para cada archivo:
     - Extraer metadata (nombre, tamaño, fecha, extensión)
     - Upsert en SQLite
   - Detectar archivos eliminados (en DB pero no en filesystem)
   - Marcar como deleted o eliminar
3. Emitir evento de progreso

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
async fn reindex_path(path: Option<String>) -> Result<(), Error>

#[tauri::command]
async fn get_indexing_status() -> Result<IndexingStatus, Error>

#[tauri::command]
async fn get_config() -> Result<SearchConfig, Error>

#[tauri::command]
async fn update_config(config: SearchConfig) -> Result<(), Error>

#[tauri::command]
async fn open_location(path: String) -> Result<(), Error>
```

**Events**:
```rust
emit!("indexing-started");
emit!("indexing-progress", progress);
emit!("indexing-completed");
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
- Parallel indexing con rayon/tokio::spawn
- Mutex/RwLock para estado compartido
- React state management (Zustand) para estado UI

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
