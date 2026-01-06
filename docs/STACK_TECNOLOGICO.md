# Stack Tecnológico - OxI Search

## 1. Core (Rust - Tauri Backend)

### 1.1 Core Dependencies

```toml
[package]
name = "oxi-search"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tauri = { version = "2.0", features = ["shell-open"] }
```

**Uso**:
- `tokio`: Async runtime para I/O concurrente
- `serde`: Serialización/deserialización
- `serde_json`: JSON parsing
- `tauri`: Desktop app framework con Rust backend

### 1.2 Filesystem & Indexing

```toml
walkdir = "2.4"
ignore = "0.4"
glob = "0.3"
rayon = "1.8"
```

**Uso**:
- `walkdir`: Recursión eficiente de directorios
- `ignore`: Patrones .gitignore-like para exclusión
- `glob`: Pattern matching de archivos
- `rayon`: Parallel processing para indexado

### 1.3 Database

```toml
rusqlite = { version = "0.30", features = ["bundled"] }
```

**Uso**:
- `rusqlite`: SQLite embedded (síncrono)

### 1.4 Search & Algorithms

```toml
fuzzy-matcher = "0.3"
regex = "1.10"
levenshtein = "1.0"
```

**Uso**:
- `fuzzy-matcher`: Búsqueda fuzzy
- `regex`: Soporte regex
- `levenshtein`: Distancia Levenshtein para fuzzy search

### 1.5 Utility

```toml
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = "0.4"
human-size = "0.4"
```

**Uso**:
- `anyhow`: Error handling
- `thiserror`: Custom error types
- `tracing`: Structured logging
- `chrono`: Fechas y tiempos
- `human-size`: Formato humano de tamaños

## 2. Frontend (React - Tauri UI)

### 2.1 Core

```json
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "typescript": "^5.3.0",
    "@tauri-apps/api": "^2.0.0"
  }
}
```

### 2.2 UI Components

```json
{
  "dependencies": {
    "@radix-ui/react-dialog": "^1.0.5",
    "@radix-ui/react-dropdown-menu": "^2.0.6",
    "@radix-ui/react-select": "^2.0.0",
    "@radix-ui/react-toast": "^1.1.5",
    "lucide-react": "^0.300.0",
    "cmdk": "^0.2.0"
  }
}
```

**Uso**:
- Componentes accesibles de Radix UI
- Iconos Lucide
- `cmdk`: Command palette para Ctrl+K

### 2.3 Styling

```json
{
  "dependencies": {
    "tailwindcss": "^3.4.0",
    "@tailwindcss/forms": "^0.5.7",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.0.0",
    "tailwind-merge": "^2.2.0"
  }
}
```

**Uso**:
- Tailwind CSS para estilos
- Utilities para class management

### 2.4 State Management

```json
{
  "dependencies": {
    "zustand": "^4.4.7"
  }
}
```

**Uso**:
- `zustand`: State manager ligero

### 2.5 Development

```json
{
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "vite": "^5.0.8",
    "@vitejs/plugin-react": "^4.2.1"
  }
}
```

## 3. Database (SQLite - Local)

### 3.1 SQLite Schema

**Ventajas**:
- Zero-configuration
- Embeddable (archivo único en disco)
- Cross-platform
- Rápido para queries simples
- Ideal para aplicaciones desktop standalone

**Ubicación**:
- `~/.oxi-search/data/oxi-search.db` (Linux/macOS)
- `%APPDATA%\OxI-Search\data\oxi-search.db` (Windows)

### 3.2 Tablas

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
```

## 4. DevOps

### 4.1 Build Tools

**Backend**:
- `cargo`: Package manager y build tool
- `cargo-nextest`: Runner de tests
- `cargo-watch`: Watch mode durante desarrollo

**Frontend**:
- `vite`: Build tool rápido
- `tauri-cli`: Package de desktop app

### 4.2 Testing

```toml
[dev-dependencies]
tokio-test = "0.4"
criterion = "0.5"
tempfile = "3.8"
```

**Frontend**:
```json
{
  "devDependencies": {
    "vitest": "^1.0.4",
    "@testing-library/react": "^14.1.2",
    "playwright": "^1.40.0"
  }
}
```

### 4.3 CI/CD

- GitHub Actions
- Scripts de build multi-platform
- Automated testing
- Release automation

## 5. Deployment

### 5.1 Packaging

**Windows**:
- `.exe` standalone
- `.msi` installer (opcional)

**Linux**:
- `.AppImage` universal
- `.deb` (opcional)

**macOS**:
- `.dmg` disk image
- `.app` bundle

### 5.2 Installation

- Instalación simple: extraer y ejecutar
- Configuración guardada en `~/.oxi-search/`
- Sin dependencias externas

## 6. Alternativas Consideradas

### UI Frameworks

| Framework | Pros | Cons |
|-----------|------|------|
| Tauri | Ligero, web skills, cross-platform, standalone | Bundle más grande |
| Iced | 100% Rust, nativo | Menos ecosistema |
| Slint | Nativo, declarativo | Comunidad pequeña |
| Electron | Maduro, ecosistema | Muy pesado, lento |

**Elección**: Tauri por balance de rendimiento y desarrollo

### Search Engine

| Librería | Pros | Cons |
|----------|------|------|
| SQLite + LIKE | Simple, rápido | Sin FTS5 avanzado |
| Meilisearch | Poderoso | Requiere servidor |
| Tantivy | Rápido | Complejidad |
| Ripgrep | Muy rápido | Solo búsqueda, no indexado |

**Elección**: SQLite con LIKE + fuzzy-matcher para simplicidad y self-contained

### Database

| DB | Pros | Cons |
|----|------|------|
| SQLite | Embeddable, rápido, zero-config | No multi-user |
| DuckDB | Analítico | Menos general |
| LMDB | Ultrarrápido | API compleja en C |

**Elección**: SQLite para simplicidad y self-contained

## 7. Estructura del Proyecto

```
oxi-search/
├── src-tauri-search/          # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands.rs        # Tauri commands
│   │   ├── indexer/
│   │   │   ├── mod.rs
│   │   │   ├── core.rs
│   │   │   ├── incremental.rs
│   │   │   └── exclusions.rs
│   │   ├── searcher/
│   │   │   ├── mod.rs
│   │   │   ├── exact.rs
│   │   │   ├── fuzzy.rs
│   │   │   └── ranking.rs
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   └── schema.rs
│   │   └── cache/
│   │       ├── mod.rs
│   │       └── manager.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src-search/                # React frontend
│   ├── App.tsx
│   ├── components/
│   │   ├── Search/
│   │   ├── Results/
│   │   └── Settings/
│   ├── lib/
│   │   ├── stores/
│   │   └── utils/
│   └── styles/
├── package.json
└── tsconfig.json
```
