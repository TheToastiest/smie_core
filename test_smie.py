from smie_core import (
    cache_semantic_memory,
    recall_semantic_memory,
    search_contexts_by_token,
    start_scanner,
    stop_scanner
)

start_scanner()

cache_semantic_memory("test", "Banana boat sails today", 60)
print("🔍", search_contexts_by_token("test", "banana", 10))

# Give Redis a second to index it
import time; time.sleep(1)

print("🔁 Full Recall:", recall_semantic_memory("test"))
stop_scanner()
