use anyhow::{bail, Result};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::ann::{AnnEngine, Metric};
use raggedy_anndy::flat::FlatIndex;
use raggedy_anndy::metric::Metric as RaMetric;

fn to_ra_metric(m: Metric) -> RaMetric {
    match m {
        Metric::Cosine => RaMetric::Cosine,
        Metric::L2 => RaMetric::L2,
    }
}

pub struct RagFlat {
    dim: usize,
    idx: Arc<RwLock<FlatIndex>>,
    // SMIE-side metadata:
    meta: Arc<RwLock<HashMap<u64, (u64, String)>>>, // id -> (tags, text)
    dead: Arc<RwLock<HashSet<u64>>>,                // tombstones
}

impl RagFlat {
    pub fn new(dim: usize, metric: Metric) -> Result<Self> {
        let idx = FlatIndex::new(dim, to_ra_metric(metric)); // returns Self, not Result
        Ok(Self {
            dim,
            idx: Arc::new(RwLock::new(idx)),
            meta: Arc::new(RwLock::new(HashMap::new())),
            dead: Arc::new(RwLock::new(HashSet::new())),
        })
    }
}

impl AnnEngine for RagFlat {
    fn dim(&self) -> usize { self.dim }

    fn add(&self, id: u64, vec: &[f32], tags: u64, text: &str) -> Result<()> {
        if vec.len() != self.dim {
            bail!("add: dim {} != index dim {}", vec.len(), self.dim);
        }
        self.idx.write().add(id, vec);                          // RA expects u64 id
        self.meta.write().insert(id, (tags, text.to_string())); // SMIE metadata
        Ok(())
    }

    fn search(&self, q: &[f32], top_k: usize, tag_mask: u64)
              -> Result<Vec<(u64, f32, u64, String)>>
    {
        if q.len() != self.dim {
            bail!("search: dim {} != index dim {}", q.len(), self.dim);
        }
        let hits = self.idx.read().search(q, top_k); // returns RA hits (id: u64)

        let meta = self.meta.read();
        let dead = self.dead.read();

        let mut out = Vec::with_capacity(hits.len());
        for h in hits {
            let id: u64 = h.id;
            if dead.contains(&id) { continue; }
            let (tags, text) = meta.get(&id).cloned().unwrap_or((0, String::new()));
            if (tags & tag_mask) == 0 { continue; }
            // NOTE: if your Hit type uses `distance` instead of `score`, change the field here.
            out.push((id, h.score, tags, text));
        }
        Ok(out)
    }

    fn tombstone(&self, id: u64) -> Result<()> { self.dead.write().insert(id); Ok(()) }

    fn save(&self, path: &str) -> Result<()> {
        #[derive(serde::Serialize)]
        struct Persist<'a> {
            meta: &'a HashMap<u64, (u64, String)>,
            dead: &'a HashSet<u64>,
        }
        let meta = self.meta.read();
        let dead = self.dead.read();
        let data = Persist { meta: &*meta, dead: &*dead };
        std::fs::write(format!("{path}.meta.json"), serde_json::to_string(&data)?)?;
        Ok(())
    }

    fn load(&self, path: &str) -> Result<()> {
        let p = format!("{path}.meta.json");
        if !std::path::Path::new(&p).exists() { return Ok(()); }
        #[derive(serde::Deserialize)]
        struct Persist { meta: HashMap<u64, (u64, String)>, dead: HashSet<u64> }
        let s = std::fs::read_to_string(p)?;
        let Persist { meta, dead } = serde_json::from_str(&s)?;
        *self.meta.write() = meta;
        *self.dead.write() = dead;
        Ok(())
    }
}
use crate::Embedder;            // make sure `pub trait Embedder` is defined in smie_core

pub struct HashEmbed { dim: usize }
impl HashEmbed { pub fn new(dim: usize) -> Self { Self { dim } } }

impl Embedder for HashEmbed {
    fn dim(&self) -> usize { self.dim }

    // ✅ your SMIE signature
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let mut v = vec![0f32; self.dim];
        let mut h: u64 = 0xcbf29ce484222325;
        for b in text.as_bytes() { h ^= *b as u64; h = h.wrapping_mul(0x100000001b3); }
        for i in 0..self.dim { v[i] = ((h.rotate_left((i % 13) as u32) & 0xffff) as f32) / 65535.0; }
        let n = (v.iter().map(|x| x * x).sum::<f32>()).sqrt().max(1e-9);
        for x in &mut v { *x /= n; }
        Ok(v)
    }
}