export interface SearchResult {
  path: string;
  name: string;
  extension: string | null;
  file_size: number | null;
  is_dir: boolean;
  modified_time: string;
  score: number;
}

export interface SearchResults {
  query: string;
  results: SearchResult[];
  total: number;
  page: number;
  limit: number;
}

export interface IndexingProgress {
  current_path: string;
  files_processed: number;
  total_files: number | null;
  status: string;
}