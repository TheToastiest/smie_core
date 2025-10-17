pub mod redis_backend;
pub mod sqlite_backend;
pub mod embedding;
pub mod ffi; 
pub mod faiss;
pub mod error;
pub mod pipeline;
pub use pipeline::{ingest, uriel_recall}; // <-- make them visible to binaries like smie_ui
pub mod burn_model;
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use redis_backend::*;
use sqlite_backend::*;
use error::SmieError;
use faiss::Index;
mod ann;            pub use ann::{AnnEngine, Metric};
#[cfg(feature="ra")]
mod rag_ann;        #[cfg(feature="ra")]
pub use rag_ann::RagFlat;
/// Starts the background TTL scanner
pub fn start_scanner() -> Result<(), SmieError> {
    redis_backend::start_scanner()
}

/// Stops the background TTL scanner
pub fn stop_scanner() -> Result<(), SmieError> {
    redis_backend::stop_scanner()
}

/// Cache a semantic memory entry into Redis
pub fn cache_semantic_memory(context: &str, data: &str, ttl_seconds: u64) -> Result<(), SmieError> {
    redis_backend::cache_entry(context, data, ttl_seconds)
}

/// Get recent entries from Redis since a timestamp
pub fn recall_recent_entries(context: &str, since: u64) -> Result<Vec<String>, SmieError> {
    redis_backend::recall_since(context, since)
}

/// Perform a reverse keyword query
pub fn reverse_index_query(context: &str, token: &str) -> Result<Vec<String>, SmieError> {
    redis_backend::search_by_token(context, token, 10)
}

pub fn flush_entry_to_sqlite(context: &str, data: &str, timestamp: Option<u64>) -> Result<(), SmieError> {
    sqlite_backend::store_entry(context, data, timestamp)
}

pub fn recall_all_memory(context: &str) -> Result<Vec<String>, SmieError> {
    let entries = sqlite_backend::recall_all(context)?;
    Ok(entries.into_iter().map(|e| format!("{} | {}", e.timestamp, e.data)).collect())
}

pub fn search_faiss_index(index_path: &str, embedding: Vec<f32>, top_k: usize) -> Result<Vec<i64>, SmieError> {
    let index = match Index::from_file(index_path) {
        Ok(i) => i,
        Err(_) => {
            println!("⚠️ FAISS index not found. Creating a new one...");
            let mut new_index = Index::new_flat_l2(embedding.len() as i32)
                .map_err(|e| SmieError::Other(format!("Failed to create index: {}", e)))?;
            new_index.save(index_path)?;
            new_index
        }
    };

    let labels = index.safe_search(&embedding, top_k)?;

    Ok(labels)
}


pub fn store_embedding_for_key(key: &str, embedding: Vec<f32>) -> Result<(), SmieError> {
    embedding::store_embedding(key, &embedding)
}

pub fn load_embedding_for_key(key: &str) -> Result<Vec<f32>, SmieError> {
    embedding::load_embedding(key)
}

pub fn ingest_text(context: &str, text: &str, ttl: u64, source: &str) -> Result<(), SmieError> {
    pipeline::ingest(
        context.to_string(),
        text.to_string(),
        ttl,
        source.to_string(),
    );
    Ok(())
}

pub fn generate_embedding_from_text(text: &str) -> Vec<f32> {
    burn_model::embed_text(text)
}

pub mod tags {
    pub const SHORT: u64 = 1 << 0;
    pub const LONG:  u64 = 1 << 1;
    pub const SYSTEM:u64 = 1 << 2;
    pub const USER:  u64 = 1 << 3;
    pub const LORE:  u64 = 1 << 4;
}

pub trait Embedder: Send + Sync {
    fn dim(&self) -> usize;
    fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

// Simple deterministic embedder to prove the pipe. Replace with Burn/ORT:
pub struct HashEmbed { dim: usize }
impl HashEmbed { pub fn new(dim: usize) -> Self { Self{dim} } }
impl Embedder for HashEmbed {
    fn dim(&self) -> usize { self.dim }
    fn embed(&self, s: &str) -> Result<Vec<f32>> {
        let mut v = vec![0f32; self.dim];
        let mut h: u64 = 1469598103934665603;
        for b in s.as_bytes() { h ^= *b as u64; h = h.wrapping_mul(1099511628211); }
        for i in 0..self.dim { v[i] = ((h.rotate_left((i%13) as u32) & 0xffff) as f32)/65535.0; }
        let n = (v.iter().map(|x|x*x).sum::<f32>()).sqrt().max(1e-9);
        for x in &mut v { *x /= n; }
        Ok(v)
    }
}
pub struct Smie {
    dim: usize,
    emb: Arc<dyn Embedder>,
    ann: Arc<dyn AnnEngine>,
    next_id: Mutex<u64>,   // was u32
}

pub struct SmieConfig {
    pub dim: usize,
    pub metric: Metric,
}

impl Smie {
    #[cfg(feature="ra")]
    pub fn new(cfg: SmieConfig, emb: Arc<dyn Embedder>) -> anyhow::Result<Self> {
        let ann = Arc::new(RagFlat::new(cfg.dim, cfg.metric)?);
        Ok(Self { dim: cfg.dim, emb, ann, next_id: Mutex::new(1u64) })
    }

    pub fn ingest(&self, text: &str, tags: u64) -> anyhow::Result<u64> {
        let v = self.emb.embed(text)?;
        assert_eq!(v.len(), self.dim, "embedder dim != smie dim");
        let mut g = self.next_id.lock();
        let id = *g; *g += 1;
        self.ann.add(id, &v, tags, text)?;
        Ok(id)
    }

    pub fn recall(&self, query: &str, k: usize, mask: u64)
                  -> anyhow::Result<Vec<(u64, f32, u64, String)>>
    {
        let v = self.emb.embed(query)?;
        assert_eq!(v.len(), self.dim, "embedder dim != smie dim");
        self.ann.search(&v, k, mask)
    }

    pub fn tombstone(&self, id: u64) -> anyhow::Result<()> { self.ann.tombstone(id) }
    pub fn save(&self, path: &str) -> anyhow::Result<()> { self.ann.save(path) }
    pub fn load(&self, path: &str) -> anyhow::Result<()> { self.ann.load(path) }
    // recall using a precomputed embedding vector
    pub fn recall_vec(
        &self,
        q: &[f32],
        k: usize,
        mask: u64,
    ) -> anyhow::Result<Vec<(u64, f32, u64, String)>> {
        if q.len() != self.dim {
            anyhow::bail!("recall_vec: dim {} != index dim {}", q.len(), self.dim);
        }
        self.ann.search(q, k, mask)
    }

}
