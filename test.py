import time
from smie_core import (
    uriel_cache,
    uriel_recall,
    cache_semantic_memory,
    recall_semantic_memory,
    recall_recent_entries,
    flush_expired_to_sqlite,
    reverse_index_query,
    search_contexts_by_token,
    get_memory_by_token,
    store_embedding_in_sqlite,
    load_embedding_from_sqlite,
    get_latest_redis_key,
    start_scanner,
    stop_scanner,
    search_similar_keys
)

context = "test_context"
sample_data = "The quick brown fox jumps over the lazy dog."
ttl = 5

# Start scanner
start_scanner()

print("🚀 Testing uriel_cache")
uriel_cache(context, sample_data, ttl, source="pytest")

print("🧠 Cached data with TTL =", ttl)
time.sleep(1)

print("🧾 Recalling recent entries")
recent = uriel_recall(context, None)  # Explicitly pass None
print(recent)

print("🔎 Reverse index query")
keywords = reverse_index_query(context, "fox")
print("Matched Keys:", keywords)

print("📚 Get memory by token")
mem = get_memory_by_token(context, "quick", limit=5)
print("Memories:", mem)

print("💾 Embedding store/load")
embedding = [0.1 * i for i in range(16)]
key = get_latest_redis_key(context) 
store_embedding_in_sqlite(key, embedding)

print("📤 Forcing flush after TTL expiration")
time.sleep(ttl + 1)
flush_expired_to_sqlite(context)

print("📥 Recalling from SQLite")
recalled = recall_semantic_memory(context)
print(recalled)

loaded = load_embedding_from_sqlite(key)
print("✔ Loaded embedding:", loaded)


# 🧠 Run FAISS search test
print("🔍 FAISS Search Test")
top_keys = search_similar_keys("indexes/test_context_index.faiss", embedding, 3)
print("Top K Redis Keys from FAISS:", top_keys)

# Stop scanner
stop_scanner()
