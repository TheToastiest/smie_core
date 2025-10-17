use crate::error::SmieError;
use rusqlite::{params, Connection};
use std::fs;

/// Opens the persistent embedding store (SQLite)
fn get_connection() -> Result<Connection, SmieError> {
    fs::create_dir_all("db")?;
    let conn = Connection::open("db/semantic_embeddings.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS semantic_embeddings (
            redis_key TEXT PRIMARY KEY,
            embedding BLOB NOT NULL
        )",
        [],
    )?;

    Ok(conn)
}

/// Store a `Vec<f32>` embedding to SQLite
pub fn store_embedding(redis_key: &str, embedding: &[f32]) -> Result<(), SmieError> {
    let conn = get_connection()?;

    let serialized = bincode::serialize(embedding)
        .map_err(|e| SmieError::Other(format!("bincode serialize failed: {e}")))?;

    conn.execute(
        "INSERT OR REPLACE INTO semantic_embeddings (redis_key, embedding) VALUES (?1, ?2)",
        params![redis_key, serialized],
    )?;

    println!("📥 Stored embedding for key: {redis_key}");
    Ok(())
}

/// Load a `Vec<f32>` embedding from SQLite by redis_key
pub fn load_embedding(redis_key: &str) -> Result<Vec<f32>, SmieError> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare("SELECT embedding FROM semantic_embeddings WHERE redis_key = ?1")?;
    let mut rows = stmt.query(params![redis_key])?;

    if let Some(row) = rows.next()? {
        let blob: Vec<u8> = row.get(0)?;
        let vec: Vec<f32> = bincode::deserialize(&blob)
            .map_err(|e| SmieError::Other(format!("bincode decode failed: {e}")))?;

        println!("📤 Loaded embedding for key: {redis_key}");
        Ok(vec)
    } else {
        Err(SmieError::Other(format!(
            "No embedding found for key: {redis_key}"
        )))
    }
}
