import sqlite3
import numpy as np

def load_embeddings_into_faiss(db_path="semantic_memory.sqlite"):
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    cursor.execute("SELECT redis_key, embedding FROM semantic_embeddings")
    entries = cursor.fetchall()

    if not entries:
        return None, []

    dim = len(np.frombuffer(entries[0][1], dtype=np.float32))
    index = faiss.IndexFlatL2(dim)
    keys = []

    for redis_key, blob in entries:
        emb = np.frombuffer(blob, dtype=np.float32)
        index.add(emb.reshape(1, -1))
        keys.append(redis_key)

    return index, keys
