use burn::tensor::{Tensor, backend::Backend};
use crate::burn_model::{embed_text, MyBackend, MyModel};
use crate::SemanticMemoryEntry;
use crate::store_entry;
use crate::redis_backend::{get_recent_keys}; 
use redis::{Commands, Client};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn ingest(context: String, text: String, ttl: u64, source: String) -> Result<(), String> {
    let device = <MyBackend as Backend>::Device::default();
    let input_vec = embed_text(&text);
    let embedding_dim = input_vec.len();

    let input_tensor = Tensor::<MyBackend, 1>::from_floats(input_vec.as_slice(), &device)
        .reshape([1, embedding_dim]);

    let model = MyModel::<MyBackend>::new(embedding_dim, 128, &device);
    let output = model.forward(input_tensor);

    let result_vec: Vec<f32> = output
        .into_data()
        .convert::<f32>()
        .into_vec()
        .map_err(|e| format!("Tensor conversion failed: {:?}", e))?;

    // ✅ Optional: write to SQLite for now
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Timestamp error: {:?}", e))?
        .as_secs();

    store_entry(&context, &text, Some(timestamp)).map_err(|e| format!("SQLite error: {:?}", e))?;

    println!(
        "[SMIE::Ingest] Context: {context}, Source: {source}, First Model Output: {}",
        result_vec.first().unwrap_or(&0.0)
    );

    println!("📥 Stored embedding for key: burn:{context}");

    Ok(())
}

pub fn uriel_recall(context: String, _filter: Option<String>) -> Result<Vec<SemanticMemoryEntry>, String> {
    let key = format!("burn:{}", context);
    let keys = get_recent_keys(&key).map_err(|e| format!("Redis error: {}", e))?;

    let client = Client::open("redis://127.0.0.1/").map_err(|e| e.to_string())?;
    let mut con = client.get_connection().map_err(|e| e.to_string())?;

    let mut results = vec![];

    for k in keys {
        if let Ok(data) = con.get::<_, String>(&k) {
            let entry = SemanticMemoryEntry {
                context: context.clone(),
                data,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            };
            results.push(entry);
        }
    }

    Ok(results)
}