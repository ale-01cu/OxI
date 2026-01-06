# Plan de Desarrollo - OxI Search

## 1. Visión General

OxI Search es una aplicación de escritorio dedicada exclusivamente a la búsqueda rápida de archivos y carpetas en el sistema local.

## 2. Objetivos Principales

- Buscar en el sistema local en <1 segundo mediante indexación
- Indexación incremental que se actualiza automáticamente
- Búsqueda por nombre, extensión, tamaño, fecha
- Resultados ordenados por relevancia
- Abrir ubicación de archivos con click derecho
- Filtros avanzados (tipo, tamaño, rango de fechas)
- Interfaz moderna, rápida e intuitiva

## 3. Requisitos Funcionales

### Búsqueda
- Búsqueda en tiempo real mientras se escribe
- Soporte para búsqueda exacta y fuzzy (Levenshtein)
- Búsqueda por extensión de archivo
- Búsqueda por rango de tamaño
- Búsqueda por rango de fechas
- Resultados paginados con lazy loading
- Click derecho → "Abrir ubicación"
- Keyboard shortcuts (Ctrl+K para abrir buscador)

### Indexación
- Indexado recursivo de directorios
- Indexación incremental (solo archivos modificados)
- Configuración de paths a excluir (.gitignore-like)
- Indexación paralela para filesystems grandes
- Progress indication durante indexado
- Re-indexado manual opcional

### UI/UX
- Command palette (Ctrl+K)
- Dark/light mode
- Filtros colapsables
- Preview de resultados
- Historial de búsquedas
- Atajos de teclado

## 4. Requisitos No Funcionales

- **Rendimiento**: Búsqueda <1s, indexado <5s para 1TB
- **Arquitectura**: App de escritorio standalone (sin servidor)
- **Seguridad**: Todo local, sin datos en la nube
- **Cross-platform**: Windows (prioridad), Linux, macOS
- **UI/UX**: Responsive, dark mode, accesible

## 5. Stack Tecnológico

**App de escritorio**: Tauri + Rust (todo en una app)
**Frontend**: React + TypeScript
**Core**: Rust con Tokio (async)
**Database**: SQLite (local, embeddable)
**Filesystem**: walkdir, ignore, glob

## 6. Roadmap

- **Sprint 1**: Setup + Indexing Engine
- **Sprint 2**: Search Engine + Cache
- **Sprint 3**: Frontend UI
- **Sprint 4**: Testing + Refinamiento
- **Sprint 5**: Packaging + Release

## 7. Entregables

- Aplicación instalable (standalone)
- User guide
- Tests automatizados

---

## KPIs de Éxito

- **Performance**: Búsqueda <1s en 1TB indexado
- **Usability**: <3 clicks para encontrar cualquier archivo
- **Coverage**: Tests >80% de código Rust
- **Installation**: Instalación <2 minutos
