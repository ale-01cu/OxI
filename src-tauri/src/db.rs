use rusqlite::{Connection, Result};
use std::path::PathBuf;
use tracing::{error, info};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&db_path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        info!("Initializing database schema");

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS search_index (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                extension TEXT,
                file_size INTEGER,
                modified_time TEXT NOT NULL,
                last_indexed TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_search_name ON search_index(name)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_search_extension ON search_index(extension)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_search_size ON search_index(file_size)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_search_modified ON search_index(modified_time)",
            [],
        )?;

        info!("Database schema initialized");
        Ok(())
    }

    pub fn upsert_file(
        &self,
        path: &str,
        name: &str,
        extension: Option<&str>,
        file_size: Option<i64>,
        modified_time: &str,
        last_indexed: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO search_index (path, name, extension, file_size, modified_time, last_indexed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [path, name, extension, file_size, modified_time, last_indexed],
        )?;
        Ok(())
    }

    pub fn delete_file(&self, path: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM search_index WHERE path = ?1", [path])?;
        Ok(())
    }

    pub fn get_file_count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM search_index", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn get_database_size(&self) -> Result<u64> {
        let size: i64 = self
            .conn
            .query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let page_size: i64 = self
            .conn
            .query_row("PRAGMA page_size", [], |row| row.get(0))?;
        Ok((size * page_size) as u64)
    }

    pub fn search_files(
        &self,
        query: &str,
        extensions: Option<Vec<String>>,
        min_size: Option<i64>,
        max_size: Option<i64>,
        limit: usize,
    ) -> Result<Vec<(String, String, Option<String>, Option<i64>, String)>> {
        let mut sql = "SELECT path, name, extension, file_size, modified_time FROM search_index WHERE name LIKE ?1".to_string();
        let mut params: Vec<&dyn rusqlite::ToSql> = vec![&format!("%{}%", query)];

        if let Some(exts) = extensions {
            if !exts.is_empty() {
                let placeholders: Vec<String> = exts.iter().map(|_| "?".to_string()).collect();
                sql.push_str(&format!(" AND extension IN ({})", placeholders.join(", ")));
                exts.iter().for_each(|ext| params.push(ext));
            }
        }

        if let Some(min) = min_size {
            sql.push_str(" AND file_size >= ?");
            params.push(&min);
        }

        if let Some(max) = max_size {
            sql.push_str(" AND file_size <= ?");
            params.push(&max);
        }

        sql.push_str(" ORDER BY name ASC LIMIT ?");
        params.push(&(limit as i64));

        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params.as_slice())?;

        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            results.push((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ));
        }

        Ok(results)
    }

    pub fn get_last_indexed_time(&self) -> Result<Option<String>> {
        let result: Option<String> = self
            .conn
            .query_row("SELECT MAX(last_indexed) FROM search_index", [], |row| {
                row.get(0)
            })
            .ok();
        Ok(result)
    }

    pub fn delete_stale_entries(&self, older_than_hours: i64) -> Result<usize> {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(older_than_hours);
        let cutoff_str = cutoff.to_rfc3339();

        let result = self.conn.execute(
            "DELETE FROM search_index WHERE last_indexed < ?1",
            [&cutoff_str],
        )?;

        Ok(result as usize)
    }

    pub fn vacuum(&self) -> Result<()> {
        self.conn.execute("VACUUM", [])?;
        Ok(())
    }

    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }
}
