use anyhow::Result;

#[derive(Clone, Copy)]
pub enum Metric { Cosine, L2 }

pub trait AnnEngine: Send + Sync {
    fn dim(&self) -> usize;
    fn add(&self, id: u64, vec: &[f32], tags: u64, text: &str) -> Result<()>;
    fn search(&self, q: &[f32], top_k: usize, tag_mask: u64)
              -> Result<Vec<(u64, f32, u64, String)>>;
    fn tombstone(&self, id: u64) -> Result<()>;
    fn save(&self, path: &str) -> Result<()>;
    fn load(&self, path: &str) -> Result<()>;
}
