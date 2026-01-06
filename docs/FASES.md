# Fases de Desarrollo - OxI Search

## Timeline Estimado: 5 semanas

---

## FASE 1: Foundation & Setup (Semana 1)

### Objetivos
- Configurar estructura del proyecto
- Setup de herramientas de desarrollo
- Primeros componentes del indexer

### Tareas

#### Backend Setup
- [ ] Inicializar proyecto Tauri (oxi-search)
- [ ] Configurar Cargo.toml con dependencias base
- [ ] Setup de Rust toolchain (stable)
- [ ] Configurar linters (clippy, rustfmt)

#### Frontend Setup
- [ ] Inicializar proyecto React + TypeScript
- [ ] Configurar Vite
- [ ] Setup de Tailwind CSS
- [ ] Setup de cmdk (command palette)
- [ ] Estructura de carpetas (components, lib, stores, pages)

#### Database
- [ ] Crear schema SQLite inicial (search_index)
- [ ] Implementar módulo de conexión a DB
- [ ] Setup de migraciones
- [ ] Tests de conexión

#### Indexer Base
- [ ] Implementar indexer básico con walkdir
- [ ] Metadata extraction (nombre, tamaño, fecha, extensión)
- [ ] Database upsert logic
- [ ] Tests de indexado

### Entregables
- Proyecto compilable
- Database funcional
- Indexer básico funcionando

---

## FASE 2: Indexing Engine (Semana 2)

### Objetivos
- Indexación completa y eficiente
- Indexación incremental
- Manejo de exclusiones

### Tareas

#### Indexing Features
- [ ] Parallel indexing con rayon
- [ ] Indexado incremental (detectar cambios)
- [ ] Detección de archivos eliminados
- [ ] Exclusiones con .gitignore-like patterns
- [ ] Progress reporting

#### Configuración
- [ ] Config file parsing (TOML)
- [ ] Paths configurables
- [ ] Exclusion patterns configurables
- [ ] Schedule de indexado (manual, hourly, daily)

#### Tauri Commands
- [ ] `reindex_path(path: Option<String>)`
- [ ] `get_indexing_status()`
- [ ] `get_config()`
- [ ] `update_config(config)`
- [ ] Events: `indexing-started`, `indexing-progress`, `indexing-completed`

#### Testing
- [ ] Unit tests para indexer
- [ ] Tests con filesystems grandes (>100k archivos)
- [ ] Tests de indexación incremental
- [ ] Performance benchmarks

### Entregables
- Indexer completo y optimizado
- Indexación incremental funcional
- Configuración flexible

---

## FASE 3: Search Engine (Semana 3)

### Objetivos
- Búsqueda por nombre
- Búsqueda fuzzy
- Filtros avanzados

### Tareas

#### Search Logic
- [ ] Búsqueda exacta por nombre (LIKE)
- [ ] Búsqueda fuzzy con Levenshtein
- [ ] Búsqueda regex
- [ ] Ranking algorithm por relevancia

#### Filters
- [ ] Filtrado por extensión
- [ ] Filtrado por rango de tamaño
- [ ] Filtrado por rango de fechas
- [ ] Combinación de filtros

#### Cache
- [ ] In-memory cache (LRU)
- [ ] TTL por defecto (1 hora)
- [ ] Invalidación al reindexar
- [ ] Prefetch de queries comunes

#### Tauri Commands
- [ ] `search_files(query, filters, page, limit)`
- [ ] `get_search_stats()`

#### Database Optimization
- [ ] Crear índices necesarios
- [ ] Optimizar queries con EXPLAIN QUERY PLAN
- [ ] Benchmarking de queries

#### Testing
- [ ] Unit tests para searcher
- [ ] Tests de diferentes tipos de búsqueda
- [ ] Tests de filtros
- [ ] Performance tests (<1s para cualquier query)

### Entregables
- Search engine completo
- Todos los tipos de búsqueda funcionales
- Caché optimizado

---

## FASE 4: Frontend UI (Semana 4)

### Objetivos
- Interfaz gráfica completa
- Integración con backend
- UX pulida

### Tareas

#### Layout
- [ ] Command palette con cmdk (Ctrl+K)
- [ ] Search input principal
- [ ] Results list con infinite scroll
- [ ] Filters panel colapsable
- [ ] Settings page
- [ ] Theme toggle (light/dark)
- [ ] Responsive design

#### Search Page
- [ ] Input con debounce
- [ ] Loading states y skeletons
- [ ] Results con iconos por tipo de archivo
- [ ] Context menu (abrir ubicación)
- [ ] Filters activos con badges
- [ ] Empty states informativos
- [ ] Error handling con toasts

#### Results Display
- [ ] File list con metadata (nombre, path, tamaño, fecha)
- [ ] Iconos por extensión
- [ ] Preview opcional
- [ ] Click derecho → "Open location"
- [ ] Keyboard navigation (↑↓ Enter)

#### Settings Page
- [ ] Paths to index (add/remove)
- [ ] Exclusion patterns (textarea)
- [ ] Indexing schedule selector
- [ ] Cache settings (toggle, TTL)
- [ ] Theme selector
- [ ] Indexing status (last indexed, files count)

#### Integration
- [ ] Llamadas a Tauri commands
- [ ] Escuchar eventos de indexación
- [ ] State management con Zustand
- [ ] Debouncing en search input
- [ ] Keyboard shortcuts

#### UX Polish
- [ ] Animaciones suaves
- [ ] Transitions
- [ ] Hover effects
- [ ] Focus states
- [ ] Loading spinners
- [ ] Toast notifications

### Entregables
- UI completa y funcional
- UX pulida
- Integración completa con backend

---

## FASE 5: Testing & Packaging (Semana 5)

### Objetivos
- Tests completos
- Performance optimization
- Packaging multi-platform

### Tareas

#### Testing
- [ ] Unit tests (80%+ coverage de Rust)
- [ ] Integration tests para Tauri commands
- [ ] E2E tests con Playwright
- [ ] Manual testing cross-platform

#### Performance
- [ ] Profiling con flamegraph
- [ ] Optimizar queries SQL
- [ ] Optimizar indexado paralelo
- [ ] Memory profiling (no leaks)
- [ ] Lazy loading en UI

#### Bug Fixes
- [ ] Fix edge cases (paths especiales, unicode)
- [ ] Fix race conditions
- [ ] Fix memory leaks
- [ ] Fix UI responsive issues

#### Documentation
- [ ] User guide
- [ ] README actualizado
- [ ] Troubleshooting guide

#### Packaging
- [ ] Windows .exe standalone
- [ ] Linux AppImage
- [ ] macOS .dmg y .app bundle
- [ ] GitHub Actions workflow
- [ ] Release automation

### Entregables
- Suite de tests completa
- Performance optimizado
- Instaladores multi-platform
- Release v0.1.0

---

## Milestones

| Milestone | Fecha | Descripción |
|-----------|-------|-------------|
| M1 | Semana 2 | Indexing engine completo |
| M2 | Semana 3 | Search engine funcional |
| M3 | Semana 4 | UI completa |
| M4 | Semana 5 | Release v0.1.0 |

---

## KPIs de Éxito

- **Performance**: Búsqueda <1s en 1TB indexado
- **Usability**: <3 clicks para encontrar cualquier archivo
- **Coverage**: Tests >80% de código Rust
- **Installation**: Instalación <2 minutos (solo descomprimir)
- **Memory**: <200MB RAM en idle

---

## Riesgos y Mitigación

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|--------------|---------|------------|
| Performance en filesystems grandes | Media | Alto | Indexado incremental, parallel processing |
| Memory leaks en long-running | Baja | Alto | Profiling regular, memory limits |
| Fuzzy search lento | Media | Medio | Cache, LRU, optimización de algoritmo |
| Detección incorrecta de cambios | Baja | Medio | Robust incremental logic con checksums |

---

## Recursos Necesarios

- **Herramientas**: Rust toolchain, Node.js, Git
- **Hardware**: Dev machine con >=8GB RAM
- **Testing**: Filesystem grande (>100k archivos)
- **Devices**: Windows y Linux para testing cross-platform
