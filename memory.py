from sentence_transformers import SentenceTransformer
import smie_core
import json

embed_model = SentenceTransformer("all-MiniLM-L6-v2")

def cache_semantic_memory_with_embedding(context, data, ttl=300):
    # Step 1: Embed text
    embedding = embed_model.encode(data).tolist()

    # Step 2: Store text & TTL
    smie_core.cache_semantic_memory(context, data, ttl)

    # Step 3: Retrieve latest Redis key (e.g., smie:test:17540...)
    latest_key = smie_core.get_latest_redis_key(context)

    # Step 4: Store embedding vector under embedding:{redis_key}
    smie_core.store_embedding(context, latest_key, json.dumps(embedding))

    print("✅ Embedded and stored semantic memory.")

    def semantic_search(query, index, keys, top_k=5):
        model = SentenceTransformer('all-MiniLM-L6-v2')
        emb = model.encode([query]).astype(np.float32)

        D, I = index.search(emb, top_k)
        return [keys[i] for i in I[0]]