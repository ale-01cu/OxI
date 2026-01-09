import { Search, X, XCircle, FileText } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { SearchResult, SearchResults } from "../types";

const SearchViewSimple = () => {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);

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

  const openLocation = async (path: string) => {
    try {
      await invoke("open_location", { path });
    } catch (error) {
      console.error("Failed to open location:", error);
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
    <main className="w-full flex justify-center items-center h-full">
      <div className="">
        <div className="w-full h-4 flex items-center justify-end relative z-0">
          <button className="bg-zinc-900/50 rounded-full p-2">
            <XCircle className="size-5 text-zinc-500"/>
          </button>
        </div>
        <div className="relative group w-lg">
          <Search className="absolute left-4 top-1/2 transform -translate-y-1/2 w-5 h-5 text-zinc-600 group-focus-within:text-orange-800 transition-colors" />
          <input
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              handleSearch(e.target.value);
            }}
            placeholder="Buscar en el sistema..."
            className="w-full pl-12 pr-4 py-2 text-lg border border-zinc-800 rounded-full bg-zinc-900/50 text-zinc-100 placeholder-zinc-600 focus:ring-2 focus:ring-orange-900/50 focus:border-orange-900/50 focus:bg-zinc-900 outline-none transition-all shadow-2xl z-10"
            autoFocus
          />
          <button className="bg-zinc-900/50 absolute flex justify-center items-center rounded-full top-0 right-0 h-full p-4">
            <X className="size-5 text-zinc-500"/>
          </button>
        </div>

        {isSearching && (
          <div className="text-center py-10">
            <div className="relative inline-block">
              <div className="w-10 h-10 border-2 border-orange-900/20 border-t-orange-800 rounded-full animate-spin"></div>
            </div>
            <p className="mt-3 text-zinc-600 font-medium tracking-wide uppercase text-xs">Buscando...</p>
          </div>
        )}

        {!isSearching && results.length > 0 && (
          <div className="bg-zinc-900/30 rounded-xl shadow-2xl border border-zinc-800/50 overflow-hidden backdrop-blur-sm mt-4">
            <div className="px-4 py-3 border-b border-zinc-800/50 bg-zinc-900/50 flex justify-between items-center">
              <h2 className="text-[10px] font-bold text-zinc-500 uppercase tracking-widest">
                {results.length} Coincidencias
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
                      <FileText className="w-5 h-5 text-zinc-500 group-hover:text-orange-800 transition-colors" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <h3 className="text-sm font-semibold text-zinc-200 truncate group-hover:text-white transition-colors">
                          {result.name}
                        </h3>
                        {result.extension && (
                          <span className="px-1.5 py-0.5 text-[10px] font-bold bg-zinc-800 text-zinc-500 rounded uppercase">
                            {result.extension.replace('.', '')}
                          </span>
                        )}
                      </div>
                      <div className="text-xs text-zinc-600 truncate font-mono mb-3">
                        {result.path}
                      </div>
                      <div className="flex items-center gap-6 text-[10px] font-medium uppercase tracking-wider">
                        <span className="text-zinc-500 bg-zinc-800/30 px-2 py-0.5 rounded">{formatFileSize(result.file_size)}</span>
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

        {!isSearching && query && results.length === 0 && (
          <div className="text-center py-10 bg-zinc-900/20 rounded-xl border border-dashed border-zinc-800 mt-4">
            <Search className="w-10 h-10 text-zinc-800 mx-auto mb-3" />
            <h3 className="text-sm font-bold text-zinc-400 uppercase tracking-widest mb-1">
              Sin resultados
            </h3>
            <p className="text-xs text-zinc-600 font-medium">
              No se encontraron archivos que coincidan con "{query}"
            </p>
          </div>
        )}
      </div>
    </main>
  );
};

export default SearchViewSimple;
