# OxI Search - Documentación

Documentación para OxI Search - Buscador ultrarrápido de archivos.

## 📁 Documentación

1. [Plan de Desarrollo](./PLAN_DESARROLLO.md)
2. [Arquitectura](./ARQUITECTURA.md)
3. [Stack Tecnológico](./STACK_TECNOLOGICO.md)
4. [Fases de Desarrollo](./FASES.md)

## 🎯 Objetivos

OxI Search es una aplicación de escritorio independiente dedicada exclusivamente a:
- Búsqueda ultrarrápida de archivos y carpetas en el sistema local
- **Auto-indexing automático** al iniciar la aplicación por primera vez
- **Detección automática de discos** y filesystems montados
- Indexación completa y secuencial para máxima estabilidad
- Abrir ubicación de archivos directamente desde resultados
- Interfaz moderna e intuitiva

## 🚀 Quick Start

```bash
# Entrar al directorio
cd OxI

# Instalar dependencias
npm install

# Ejecutar en desarrollo
npm run tauri dev

# Build para producción
npm run tauri build
```

## 🏗️ Arquitectura

```
┌─────────────────────────────────────────────────────┐
│              Interfaz (React + TypeScript)        │
│  - Search Input                                     │
│  - Results List                                     │
│  - Filters (tipo, tamaño, fecha)                   │
│  - Indexing Progress Indicator                      │
├─────────────────────────────────────────────────────┤
│              Tauri Bridge                          │
│  - Commands: search_files, reindex_path             │
│  - Events: indexing-progress, indexing-completed    │
│  - Auto-indexing on startup                         │
├─────────────────────────────────────────────────────┤
│         Core Rust                                   │
│  - Search Engine (indexer, searcher, cache)        │
│  - File Indexing (walkdir, ignore patterns)        │
│  - Auto disk detection (/proc/mounts, drives)      │
│  - Sequential indexing (no SQLite conflicts)      │
├─────────────────────────────────────────────────────┤
│              SQLite (local)                        │
│  - search_index (caché de archivos indexados)      │
└─────────────────────────────────────────────────────┘
```

## 📊 Stack

| Componente | Tecnología |
|------------|------------|
| Desktop App | Tauri |
| Core | Rust + Tokio |
| Frontend | React + TypeScript |
| Database | SQLite (embeddable) |
| Filesystem | walkdir, ignore |

## 📈 Roadmap

- **Sprint 1**: Setup + Indexing Engine
- **Sprint 2**: Search Engine + Cache
- **Sprint 3**: Frontend UI
- **Sprint 4**: Testing + Refinamiento
- **Sprint 5**: Packaging

---

**Versión**: 0.1.0-alpha
