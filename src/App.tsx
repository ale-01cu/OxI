import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Search, HardDrive, Settings, FileText, Folder, Minus, Square, X } from "lucide-react";

interface SearchResult {
  path: string;
  name: string;
  extension: string | null;
  file_size: number | null;
  is_dir: boolean;
  modified_time: string;
  score: number;
}

interface SearchResults {
  query: string;
  results: SearchResult[];
  total: number;
  page: number;
  limit: number;
}

interface IndexingProgress {
  current_path: string;
  files_processed: number;
  total_files: number | null;
  status: string;
}

function App() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [isIndexing, setIsIndexing] = useState(false);
  const [indexingProgress, setIndexingProgress] = useState<IndexingProgress | null>(null);
  const [totalFiles, setTotalFiles] = useState(0);

  useEffect(() => {
    loadIndexingStatus();

    const unlistenProgress = listen<IndexingProgress>("indexing-progress", (event) => {
      setIndexingProgress(event.payload);
      setIsIndexing(true);
    });

    const unlistenCompleted = listen<number>("indexing-completed", (event) => {
      setTotalFiles(event.payload);
      setIsIndexing(false);
      setIndexingProgress(null);
      loadIndexingStatus();
    });

    return () => {
      unlistenProgress.then(f => f());
      unlistenCompleted.then(f => f());
    };
  }, []);

  const loadIndexingStatus = async () => {
    try {
      const status = await invoke<any>("get_indexing_status");
      setTotalFiles(status.total_files || 0);
    } catch (error) {
      console.error("Failed to load indexing status:", error);
    }
  };

  const handleSearch = async (searchQuery: string) => {
    if (!searchQuery.trim()) {
      setResults([]);
      return;
    }

    setIsSearching(true);
    try {
      const response: SearchResults = await invoke("search_files", {
        query: searchQuery,
        filters: {
          extensions: null,
          minSize: null,
          maxSize: null,
          minDate: null,
          maxDate: null,
        },
        page: 0,
        limit: 50,
      });
      setResults(response.results);
    } catch (error) {
      console.error("Search failed:", error);
    } finally {
      setIsSearching(false);
    }
  };

  const startIndexing = async () => {
    try {
      await invoke("reindex_path", {
        path: null,
        excludePatterns: [],
      });
    } catch (error) {
      console.error("Failed to start indexing:", error);
    }
  };

  const openLocation = async (path: string) => {
    try {
      await invoke("open_location", { path });
    } catch (error) {
      console.error("Failed to open location:", error);
    }
  };

  const minimizeWindow = async () => {
    try {
      await invoke("minimize_window");
    } catch (error) {
      console.error("Failed to minimize window:", error);
    }
  };

  const toggleMaximizeWindow = async () => {
    try {
      await invoke("toggle_maximize_window");
    } catch (error) {
      console.error("Failed to toggle maximize window:", error);
    }
  };

  const closeWindow = async () => {
    try {
      await invoke("close_window");
    } catch (error) {
      console.error("Failed to close window:", error);
    }
  };

  const handleDoubleClickTitleBar = async () => {
    await toggleMaximizeWindow();
  };

  const startDragging = async (e: React.MouseEvent) => {
    if (e.button === 0) {
      try {
        await invoke("start_dragging");
      } catch (error) {
        console.error("Failed to start dragging:", error);
      }
    }
  };

  const formatFileSize = (bytes: number | null) => {
    if (!bytes) return "-";
    const units = ["B", "KB", "MB", "GB"];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(1)} ${units[unitIndex]}`;
  };

  const formatDate = (dateStr: string) => {
    try {
      return new Date(dateStr).toLocaleDateString();
    } catch {
      return "-";
    }
  };

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-300 border-4 border-zinc-900/50 border-solid">
       <nav
        data-tauri-drag-region
        onMouseDown={startDragging}
        onDoubleClick={handleDoubleClickTitleBar}
        className="border-b border-zinc-800 bg-zinc-900/50 backdrop-blur-sm sticky top-0 z-10 select-none cursor-move py-2"
      >
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16 items-center">
            {/* Logo y Nombre - También actúan como zona de arrastre */}
            <div className="flex items-center gap-3 flex-1" data-tauri-drag-region>
              <div className="p-2 bg-orange-950/30 rounded-lg border border-orange-900/30 pointer-events-none">
                <Search className="w-6 h-6 text-orange-700" />
              </div>
              <div className="flex flex-col justify-center pointer-events-none">
                <span className="text-xl font-bold text-zinc-100 tracking-tight leading-none">
                  OxI <span className="text-orange-800">Search</span>
                </span>
                <span className="text-[10px] text-zinc-600 font-medium uppercase tracking-[0.2em] mt-1">
                  Engine v1.0
                </span>
              </div>
            </div>

            {/* Elementos Interactivos - Detenemos la propagación del drag aquí si es necesario, 
                pero Tauri maneja bien los botones dentro de drag regions */}
            <div className="flex items-center gap-4">
              <div className="hidden sm:block text-[10px] text-zinc-500 font-mono bg-zinc-950/50 px-2 py-1 rounded border border-zinc-800/50">
                {totalFiles.toLocaleString()} ARCHIVOS
              </div>
              
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  startIndexing();
                }}
                disabled={isIndexing}
                className="flex items-center gap-2 px-4 py-2 bg-orange-900/20 text-orange-500 border border-orange-800/30 rounded-md hover:bg-orange-900/40 disabled:opacity-50 transition-all duration-200 font-medium text-sm"
              >
                <HardDrive className="w-4 h-4" />
                {isIndexing ? "Indexando..." : "Reindexar"}
              </button>

              <button className="p-2 text-zinc-500 hover:bg-zinc-800 rounded-md transition-colors">
                <Settings className="w-5 h-5" />
              </button>

              <div className="flex items-center gap-1 border-l border-zinc-800 ml-2 pl-3">
                <button
                  onClick={minimizeWindow}
                  className="p-1.5 text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800 rounded transition-all"
                >
                  <Minus className="w-4 h-4" />
                </button>
                <button
                  onClick={toggleMaximizeWindow}
                  className="p-1.5 text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800 rounded transition-all"
                >
                  <Square className="w-3.5 h-3.5" />
                </button>
                <button
                  onClick={closeWindow}
                  className="p-1.5 text-zinc-500 hover:text-white hover:bg-red-950/50 rounded transition-all"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </nav>

      {isIndexing && indexingProgress && (
        <div className="border-b border-orange-900/30 bg-orange-950/10">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-2">
            <div className="flex items-center gap-3">
              <div className="animate-spin">
                <HardDrive className="w-4 h-4 text-orange-800" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="text-[10px] uppercase tracking-widest font-bold text-orange-900">
                  Indexando Archivos
                </div>
                <div className="text-xs text-orange-700/80 truncate font-mono">
                  {indexingProgress.current_path}
                </div>
              </div>
              <div className="text-xs font-mono text-orange-800">
                {indexingProgress.files_processed}
              </div>
            </div>
          </div>
        </div>
      )}

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="mb-8">
          <div className="relative group">
            <Search className="absolute left-4 top-1/2 transform -translate-y-1/2 w-5 h-5 text-zinc-600 group-focus-within:text-orange-800 transition-colors" />
            <input
              type="text"
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                handleSearch(e.target.value);
              }}
              placeholder="Buscar en el sistema..."
              className="w-full pl-12 pr-4 py-4 text-lg border border-zinc-800 rounded-xl bg-zinc-900/50 text-zinc-100 placeholder-zinc-600 focus:ring-2 focus:ring-orange-900/50 focus:border-orange-900/50 focus:bg-zinc-900 outline-none transition-all shadow-2xl"
              autoFocus
            />
          </div>
        </div>

        {isSearching && (
          <div className="text-center py-20">
            <div className="relative inline-block">
              <div className="w-12 h-12 border-2 border-orange-900/20 border-t-orange-800 rounded-full animate-spin"></div>
            </div>
            <p className="mt-4 text-zinc-600 font-medium tracking-wide uppercase text-xs">Buscando coincidencias...</p>
          </div>
        )}

        {!isSearching && results.length > 0 && (
          <div className="bg-zinc-900/30 rounded-xl shadow-2xl border border-zinc-800/50 overflow-hidden backdrop-blur-sm">
            <div className="px-4 py-3 border-b border-zinc-800/50 bg-zinc-900/50 flex justify-between items-center">
              <h2 className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest">
                {results.length} Coincidencias encontradas
              </h2>
            </div>
            <ul className="divide-y divide-zinc-800/50">
              {results.map((result, index) => (
                <li
                  key={index}
                  className="px-4 py-4 hover:bg-orange-950/5 transition-colors group"
                >
                  <div className="flex items-start gap-4">
                    <div className="p-2 bg-zinc-800/50 rounded group-hover:bg-orange-900/20 transition-colors">
                      {result.is_dir ? (
                        <Folder className="w-5 h-5 text-orange-700 group-hover:text-orange-500 transition-colors" />
                      ) : (
                        <FileText className="w-5 h-5 text-zinc-500 group-hover:text-orange-800 transition-colors" />
                      )}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <h3 className="text-sm font-semibold text-zinc-200 truncate group-hover:text-white transition-colors">
                          {result.name}
                        </h3>
                        {!result.is_dir && result.extension && (
                          <span className="px-1.5 py-0.5 text-[10px] font-bold bg-zinc-800 text-zinc-500 rounded uppercase">
                            {result.extension.replace('.', '')}
                          </span>
                        )}
                      </div>
                      <div className="text-xs text-zinc-600 truncate font-mono mb-3">
                        {result.path}
                      </div>
                      <div className="flex items-center gap-6 text-[10px] font-medium uppercase tracking-wider">
                        <span className="text-zinc-500 bg-zinc-800/30 px-2 py-0.5 rounded">{result.is_dir ? "Carpeta" : formatFileSize(result.file_size)}</span>
                        <span className="text-zinc-500">{formatDate(result.modified_time)}</span>
                        <button
                          onClick={() => openLocation(result.path)}
                          className="ml-auto text-orange-800 hover:text-orange-600 font-bold transition-colors underline-offset-4 hover:underline"
                        >
                          Abrir ubicación
                        </button>
                      </div>
                    </div>
                  </div>
                </li>
              ))}
            </ul>
          </div>
        )}

        {!query && !isSearching && results.length === 0 && (
          <div className="text-center py-20 bg-zinc-900/20 rounded-xl border border-dashed border-zinc-800">
            <Search className="w-12 h-12 text-zinc-800 mx-auto mb-4" />
            <h3 className="text-sm font-bold text-zinc-400 uppercase tracking-widest mb-1">
              Buscar con OxI
            </h3>
            <p className="text-xs text-zinc-600 font-medium">
              Busca en tu sistema de archivos para encontrar archivos y carpetas.
            </p>
          </div>
        )}

        {!isSearching && query && results.length === 0 && (
          <div className="text-center py-20 bg-zinc-900/20 rounded-xl border border-dashed border-zinc-800">
            <Search className="w-12 h-12 text-zinc-800 mx-auto mb-4" />
            <h3 className="text-sm font-bold text-zinc-400 uppercase tracking-widest mb-1">
              Sin resultados
            </h3>
            <p className="text-xs text-zinc-600 font-medium">
              No se encontraron archivos que coincidan con "{query}"
            </p>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
