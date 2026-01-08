import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Search, HardDrive, Settings, FileText, Minus, Square, X } from "lucide-react";

interface SearchResult {
  path: string;
  name: string;
  extension: string | null;
  file_size: number | null;
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
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div
        data-tauri-drag-region
        onMouseDown={startDragging}
        onDoubleClick={handleDoubleClickTitleBar}
        className="h-8 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between px-4 select-none cursor-move"
      >
        <div className="flex items-center gap-2">
          <Search className="w-5 h-5 text-blue-600" />
          <span className="text-sm font-semibold">OxI Search</span>
        </div>
        <div className="text-xs text-gray-600 dark:text-gray-400">
          Doble clic para maximizar/restaurar
        </div>
      </div>
      <nav className="border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16 items-center">
            <div className="flex items-center gap-2 flex-1">
              <Search className="w-8 h-8 text-blue-600" />
              <span className="text-xl font-bold">OxI Search</span>
            </div>
            <div className="flex items-center gap-4">
              <div className="text-sm text-gray-600 dark:text-gray-400 flex-1">
                {totalFiles} archivos indexados
              </div>
              <button
                onClick={startIndexing}
                disabled={isIndexing}
                className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 transition-colors"
              >
                <HardDrive className="w-4 h-4" />
                {isIndexing ? "Indexando..." : "Reindexar"}
              </button>
              <button className="p-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md">
                <Settings className="w-6 h-6" />
              </button>
              <div className="flex items-center gap-1 border-l border-gray-300 dark:border-gray-600 pl-4">
                <button
                  onClick={minimizeWindow}
                  className="p-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md transition-colors"
                >
                  <Minus className="w-4 h-4" />
                </button>
                <button
                  onClick={toggleMaximizeWindow}
                  className="p-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md transition-colors"
                >
                  <Square className="w-4 h-4" />
                </button>
                <button
                  onClick={closeWindow}
                  className="p-2 text-gray-600 dark:text-gray-400 hover:bg-red-100 dark:hover:bg-red-900/30 rounded-md transition-colors"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </nav>

      {isIndexing && indexingProgress && (
        <div className="border-b border-blue-200 dark:border-blue-800 bg-blue-50 dark:bg-blue-900/20">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-3">
            <div className="flex items-center gap-3">
              <div className="animate-spin">
                <HardDrive className="w-5 h-5 text-blue-600" />
              </div>
              <div className="flex-1">
                <div className="text-sm font-medium text-blue-900 dark:text-blue-100">
                  Indexando: {indexingProgress.files_processed} archivos
                </div>
                <div className="text-xs text-blue-700 dark:text-blue-300 truncate max-w-2xl">
                  {indexingProgress.current_path}
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="mb-6">
          <div className="relative">
            <Search className="absolute left-4 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" />
            <input
              type="text"
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                handleSearch(e.target.value);
              }}
              placeholder="Buscar archivos..."
              className="w-full pl-12 pr-4 py-4 text-lg border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none"
              autoFocus
            />
          </div>
        </div>

        {isSearching && (
          <div className="text-center py-12">
            <div className="animate-spin inline-block w-8 h-8 border-4 border-blue-600 border-t-transparent rounded-full"></div>
            <p className="mt-4 text-gray-600 dark:text-gray-400">Buscando...</p>
          </div>
        )}

        {!isSearching && results.length > 0 && (
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden">
            <div className="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
              <h2 className="text-sm font-medium text-gray-700 dark:text-gray-300">
                {results.length} resultados encontrados
              </h2>
            </div>
            <ul className="divide-y divide-gray-200 dark:divide-gray-700">
              {results.map((result, index) => (
                <li
                  key={index}
                  className="px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                >
                  <div className="flex items-start gap-3">
                    <FileText className="w-5 h-5 text-gray-400 mt-0.5 flex-shrink-0" />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <h3 className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                          {result.name}
                        </h3>
                        <span className="text-xs text-gray-500 dark:text-gray-400 flex-shrink-0">
                          {result.extension}
                        </span>
                      </div>
                      <div className="text-xs text-gray-600 dark:text-gray-400 truncate mt-1">
                        {result.path}
                      </div>
                      <div className="flex items-center gap-4 mt-2 text-xs text-gray-500 dark:text-gray-400">
                        <span>{formatFileSize(result.file_size)}</span>
                        <span>{formatDate(result.modified_time)}</span>
                        <button
                          onClick={() => openLocation(result.path)}
                          className="text-blue-600 hover:text-blue-700 transition-colors"
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

        {!isSearching && query && results.length === 0 && (
          <div className="text-center py-12">
            <FileText className="w-16 h-16 text-gray-300 dark:text-gray-600 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
              No se encontraron resultados
            </h3>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Intenta con otra búsqueda
            </p>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
