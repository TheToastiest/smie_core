use crate::error::SmieError;
use rusqlite::{params, Connection};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct SemanticMemoryEntry {
    pub context: String,
    pub data: String,
    pub timestamp: u64,
}

fn get_connection() -> Result<Connection, SmieError> {
    let conn = Connection::open("semantic_memory.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS semantic_memory (
            context TEXT NOT NULL,
            data TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

/// Stores a new semantic memory entry (manually flushed from Redis)
pub fn store_entry(context: &str, data: &str, timestamp: Option<u64>) -> Result<(), SmieError> {
    let conn = get_connection()?;
    let ts = timestamp.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    conn.execute(
        "INSERT INTO semantic_memory (context, data, timestamp) VALUES (?1, ?2, ?3)",
        params![context, data, ts],
    )?;

    Ok(())
}

/// Retrieves all semantic memory entries for a context
pub fn recall_all(context: &str) -> Result<Vec<SemanticMemoryEntry>, SmieError> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare(
        "SELECT context, data, timestamp FROM semantic_memory WHERE context = ?1",
    )?;

    let rows = stmt.query_map([context], |row| {
        Ok(SemanticMemoryEntry {
            context: row.get(0)?,
            data: row.get(1)?,
            timestamp: row.get(2)?,
        })
    })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }

    Ok(result)
}
