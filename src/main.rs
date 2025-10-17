use smie_core::*;

fn main() {
    // Start scanner (background thread)
    if let Err(e) = start_scanner() {
        eprintln!("Failed to start scanner: {e}");
    }

    // Cache a memory entry
    if let Err(e) = cache_semantic_memory("rust_test", "All memory is stored locally now.", 10) {
        eprintln!("Cache failed: {e}");
    }

    // Sleep briefly to let it flush
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Recall entries
    match recall_recent_entries("rust_test", 0) {
        Ok(entries) => {
            println!("\n🔍 Recent Entries:");
            for line in entries {
                println!(" - {}", line);
            }
        }
        Err(e) => eprintln!("Recall failed: {e}"),
    }
        let key = "rust:demo:vec1";
    let vec = vec![1.0_f32; 384];

    smie_core::store_embedding_for_key(key, vec.clone()).unwrap();
    let loaded = smie_core::load_embedding_for_key(key).unwrap();

    println!("✅ Embedding loaded: len = {}", loaded.len());
    assert_eq!(vec, loaded);
    let context = "rust_pipeline";
    let text = "Pipeline ingestion test with Burn placeholder.";
    let ttl = 15;

    match smie_core::ingest_text(context, text, ttl, "test") {
        Ok(_) => println!("✅ Ingested successfully."),
        Err(e) => eprintln!("❌ Ingest failed: {e}"),
    }
    let text = "Example sentence to embed using burn.";
let vec = smie_core::generate_embedding_from_text(text);
println!("🔢 Generated embedding: len = {}", vec.len());

    // Stop scanner
    let _ = stop_scanner();
}
