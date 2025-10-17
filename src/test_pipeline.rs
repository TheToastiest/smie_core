// src/bin/test_pipeline.rs (or tests/test_pipeline.rs)
use smie_core::{Smie, SmieConfig, Metric, HashEmbed, tags, Embedder};
use std::sync::Arc;

#[cfg(feature = "ra")]
fn main() -> anyhow::Result<()> {
    // --- inputs ---
    let context  = "rust_pipeline";
    let sentence = "Rust is fast and memory safe without garbage collection.";
    let _ttl     = 300; // TTL handled elsewhere if you're using Redis; not used by SMIE directly.

    // --- SMIE init (raggedy_anndy backend via smie_core feature "ra") ---
    // IMPORTANT: dim must match your embedder output (you reported 384).
    let dim = 384;

    let embedder = Arc::new(HashEmbed::new(384));
    let smie = Smie::new(
        SmieConfig { dim: embedder.dim(), metric: Metric::Cosine },
        embedder.clone(),
    )?;

    println!("🚀 Ingesting...");
    // Include context/source in the stored text (simple, effective).
    let text = format!("[ctx:{}][src:test] {}", context, sentence);
    let id = smie.ingest(&text, tags::USER | tags::SHORT)?;
    println!("✅ Ingested id {}", id);

    println!("📤 Generating embedding...");
    let v = embedder.embed(sentence)?;
    println!("🔢 Generated embedding: len = {}", v.len());

    println!("🔍 Searching raggedy_anndy (Flat)...");
    // Mask selects which tagged memories are eligible; here we search everything.
    let hits = smie.recall("What is Rust?", 5, !0u64)?;
    println!("🎯 Top matches:");
    for (rank, (id, score, tags_bits, text)) in hits.into_iter().enumerate() {
        println!("  {}. id={} score={:.4} tags=0x{:x} :: {}", rank + 1, id, score, tags_bits, text);
    }

    // Uncomment if you want to persist:
    // std::fs::create_dir_all("data").ok();
    // smie.save("data/smie.idx")?;

    Ok(())
}
