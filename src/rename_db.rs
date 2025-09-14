use rusqlite::{Connection, Result as SqliteResult, params};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameRecord {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    pub directory: PathBuf,
    pub prefix_removed: String,
    pub operation_id: String, // Groups related renames together
}

#[derive(Debug, Clone)]
pub struct RenameDatabase {
    db_path: PathBuf,
}

impl RenameDatabase {
    /// Create a new database instance
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
    
    /// Get default database path (in user's home/.ftmi/renames.db)
    pub fn default_path() -> SqliteResult<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| rusqlite::Error::InvalidPath("Could not find home directory".into()))?;
        
        let ftmi_dir = PathBuf::from(home).join(".ftmi");
        if !ftmi_dir.exists() {
            fs::create_dir_all(&ftmi_dir)
                .map_err(|e| rusqlite::Error::InvalidPath(format!("Could not create .ftmi directory: {}", e).into()))?;
        }
        
        Ok(ftmi_dir.join("renames.db"))
    }
    
    /// Initialize the database with required tables
    pub fn initialize(&self) -> SqliteResult<()> {
        let conn = Connection::open(&self.db_path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS renames (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                old_path TEXT NOT NULL,
                new_path TEXT NOT NULL,
                directory TEXT NOT NULL,
                prefix_removed TEXT NOT NULL,
                operation_id TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create index for faster operation_id lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_operation_id ON renames(operation_id)",
            [],
        )?;
        
        // Create index for faster timestamp lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON renames(timestamp)",
            [],
        )?;
        
        Ok(())
    }
    
    /// Record a rename operation
    pub fn record_rename(
        &self,
        old_path: &Path,
        new_path: &Path,
        directory: &Path,
        prefix_removed: &str,
        operation_id: &str,
    ) -> SqliteResult<i64> {
        let conn = Connection::open(&self.db_path)?;
        let timestamp = Utc::now();
        
        conn.execute(
            "INSERT INTO renames (timestamp, old_path, new_path, directory, prefix_removed, operation_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                timestamp.to_rfc3339(),
                old_path.to_string_lossy(),
                new_path.to_string_lossy(),
                directory.to_string_lossy(),
                prefix_removed,
                operation_id,
            ],
        )?;
        
        Ok(conn.last_insert_rowid())
    }
    
    /// Get recent operations (last N operations)
    pub fn get_recent_operations(&self, limit: usize) -> SqliteResult<Vec<String>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT DISTINCT operation_id, MIN(timestamp) as first_timestamp
             FROM renames 
             GROUP BY operation_id 
             ORDER BY first_timestamp DESC 
             LIMIT ?1"
        )?;
        
        let operation_ids = stmt.query_map(params![limit], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut result = Vec::new();
        for operation_id in operation_ids {
            result.push(operation_id?);
        }
        
        Ok(result)
    }
    
    /// Get all renames for a specific operation
    pub fn get_operation_renames(&self, operation_id: &str) -> SqliteResult<Vec<RenameRecord>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, old_path, new_path, directory, prefix_removed, operation_id
             FROM renames 
             WHERE operation_id = ?1 
             ORDER BY timestamp ASC"
        )?;
        
        let rename_iter = stmt.query_map(params![operation_id], |row| {
            let timestamp_str: String = row.get(1)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|e| rusqlite::Error::InvalidColumnType(1, "timestamp".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            Ok(RenameRecord {
                id: row.get(0)?,
                timestamp,
                old_path: PathBuf::from(row.get::<_, String>(2)?),
                new_path: PathBuf::from(row.get::<_, String>(3)?),
                directory: PathBuf::from(row.get::<_, String>(4)?),
                prefix_removed: row.get(5)?,
                operation_id: row.get(6)?,
            })
        })?;
        
        let mut result = Vec::new();
        for record in rename_iter {
            result.push(record?);
        }
        
        Ok(result)
    }
    
    /// Undo a specific operation (reverse all renames in that operation)
    pub fn undo_operation(&self, operation_id: &str) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        let records = self.get_operation_renames(operation_id)?;
        
        let mut success_count = 0;
        let mut error_count = 0;
        
        // Reverse the renames (go backwards through the list)
        for record in records.iter().rev() {
            // Check if the "new" path still exists and the "old" path doesn't exist
            if record.new_path.exists() && !record.old_path.exists() {
                match fs::rename(&record.new_path, &record.old_path) {
                    Ok(_) => {
                        success_count += 1;
                        println!("✓ Undid: {} → {}", 
                                record.new_path.display(), 
                                record.old_path.display());
                    }
                    Err(e) => {
                        error_count += 1;
                        eprintln!("❌ Failed to undo: {} → {}: {}", 
                                 record.new_path.display(), 
                                 record.old_path.display(), 
                                 e);
                    }
                }
            } else {
                error_count += 1;
                eprintln!("⚠️  Cannot undo: {} (file state changed)", record.new_path.display());
            }
        }
        
        Ok((success_count, error_count))
    }
    
    /// Delete old records (older than specified days)
    pub fn cleanup_old_records(&self, days: u32) -> SqliteResult<usize> {
        let conn = Connection::open(&self.db_path)?;
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        
        let deleted = conn.execute(
            "DELETE FROM renames WHERE timestamp < ?1",
            params![cutoff.to_rfc3339()],
        )?;
        
        Ok(deleted)
    }
}

/// Generate a unique operation ID for grouping related renames
pub fn generate_operation_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    format!("op_{}", timestamp)
}

/// Perform a rename operation with database tracking
pub fn tracked_rename(
    db: &RenameDatabase,
    old_path: &Path,
    new_path: &Path,
    prefix_removed: &str,
    operation_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the directory (parent of the old path)
    let directory = old_path.parent()
        .ok_or("Could not determine parent directory")?;
    
    // Perform the actual rename
    fs::rename(old_path, new_path)?;
    
    // Record in database
    db.record_rename(old_path, new_path, directory, prefix_removed, operation_id)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    
    #[test]
    fn test_database_operations() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let db = RenameDatabase::new(db_path);
        
        // Initialize database
        db.initialize()?;
        
        // Create test paths
        let old_path = temp_dir.path().join("[Artist] Song.mp3");
        let new_path = temp_dir.path().join("Song.mp3");
        let directory = temp_dir.path();
        
        // Record a rename
        let operation_id = generate_operation_id();
        let record_id = db.record_rename(&old_path, &new_path, directory, "Artist", &operation_id)?;
        
        assert!(record_id > 0);
        
        // Get recent operations
        let recent = db.get_recent_operations(10)?;
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0], operation_id);
        
        // Get operation renames
        let renames = db.get_operation_renames(&operation_id)?;
        assert_eq!(renames.len(), 1);
        assert_eq!(renames[0].prefix_removed, "Artist");
        
        Ok(())
    }
    
    #[test]
    fn test_tracked_rename() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let db = RenameDatabase::new(db_path);
        db.initialize()?;
        
        // Create a test file
        let old_path = temp_dir.path().join("[Test] File.txt");
        let new_path = temp_dir.path().join("File.txt");
        File::create(&old_path)?;
        
        let operation_id = generate_operation_id();
        
        // Perform tracked rename
        tracked_rename(&db, &old_path, &new_path, "Test", &operation_id)?;
        
        // Verify file was renamed
        assert!(!old_path.exists());
        assert!(new_path.exists());
        
        // Verify database record
        let renames = db.get_operation_renames(&operation_id)?;
        assert_eq!(renames.len(), 1);
        
        Ok(())
    }
}