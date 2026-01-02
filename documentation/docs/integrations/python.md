# Python Bindings

Liath provides Python bindings via PyO3 for use in Python applications.

## Installation

### From Source

Build with the `python` feature:

```bash
cd liath-rs
cargo build --release --features python

# Install the wheel
pip install target/wheels/liath-*.whl
```

### Using maturin

```bash
pip install maturin
maturin develop --features python
```

## Basic Usage

```python
from liath import Liath

# Create database
db = Liath("./my_data")

# Store data
db.put("notes", "note:1", "Hello, world!")

# Retrieve data
value = db.get("notes", "note:1")
print(value)  # "Hello, world!"

# Delete data
db.delete("notes", "note:1")
```

## API Reference

### Liath Class

#### Constructor

```python
db = Liath(data_dir: str)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `data_dir` | str | Path to data directory |

#### put

Store a key-value pair.

```python
db.put(namespace: str, key: str, value: str) -> None
```

**Example:**

```python
db.put("users", "user:1", '{"name": "Alice"}')
```

#### get

Retrieve a value.

```python
db.get(namespace: str, key: str) -> Optional[str]
```

**Example:**

```python
value = db.get("users", "user:1")
if value:
    user = json.loads(value)
    print(user["name"])
```

#### delete

Delete a key-value pair.

```python
db.delete(namespace: str, key: str) -> None
```

#### create_namespace

Create a namespace with vector index.

```python
db.create_namespace(
    name: str,
    dimensions: int,
    metric: str = "cosine"
) -> None
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `name` | str | Namespace name |
| `dimensions` | int | Vector dimensions |
| `metric` | str | "cosine", "euclidean", or "ip" |

**Example:**

```python
db.create_namespace("documents", 384, "cosine")
```

#### list_namespaces

List all namespaces.

```python
db.list_namespaces() -> List[str]
```

#### store_with_embedding

Store text with automatic embedding.

```python
db.store_with_embedding(
    namespace: str,
    id: str,
    content: str
) -> None
```

**Example:**

```python
db.store_with_embedding(
    "docs",
    "doc:1",
    "Introduction to machine learning"
)
```

#### semantic_search

Search for similar content.

```python
db.semantic_search(
    namespace: str,
    query: str,
    k: int = 10
) -> List[Dict]
```

**Returns:** List of results with `id`, `content`, and `distance`

**Example:**

```python
results = db.semantic_search("docs", "AI algorithms", k=5)
for r in results:
    print(f"{r['id']}: {r['content'][:50]}... (dist: {r['distance']:.3f})")
```

#### generate_embedding

Generate embedding for text.

```python
db.generate_embedding(text: str) -> List[float]
```

**Example:**

```python
embedding = db.generate_embedding("Hello, world!")
print(f"Dimensions: {len(embedding)}")
```

#### execute

Execute Lua code.

```python
db.execute(code: str, user_id: str = "python") -> str
```

**Example:**

```python
result = db.execute("""
    local results = semantic_search("docs", "machine learning", 5)
    return json.encode(results)
""")
print(result)
```

#### save

Persist all data to disk.

```python
db.save() -> None
```

#### close

Close the database.

```python
db.close() -> None
```

## Examples

### Document Store

```python
from liath import Liath
import json

db = Liath("./docs_db")
db.create_namespace("documents", 384)

# Store documents
docs = [
    {"id": "doc:1", "title": "Python Basics", "content": "Python is a versatile programming language..."},
    {"id": "doc:2", "title": "Machine Learning", "content": "ML enables computers to learn from data..."},
    {"id": "doc:3", "title": "Web Development", "content": "Building websites with modern frameworks..."},
]

for doc in docs:
    db.store_with_embedding("documents", doc["id"], doc["content"])
    db.put("documents:meta", doc["id"], json.dumps({"title": doc["title"]}))

# Search
results = db.semantic_search("documents", "programming languages", k=2)

for r in results:
    meta = json.loads(db.get("documents:meta", r["id"]) or "{}")
    print(f"Title: {meta.get('title')}")
    print(f"Distance: {r['distance']:.3f}")
    print(f"Content: {r['content'][:100]}...")
    print("---")

db.close()
```

### RAG Application

```python
from liath import Liath
import json

class RAGSystem:
    def __init__(self, data_dir: str):
        self.db = Liath(data_dir)
        self.db.create_namespace("knowledge", 384)

    def add_document(self, doc_id: str, content: str, metadata: dict = None):
        self.db.store_with_embedding("knowledge", doc_id, content)
        if metadata:
            self.db.put("knowledge:meta", doc_id, json.dumps(metadata))

    def retrieve(self, query: str, k: int = 5) -> list:
        results = self.db.semantic_search("knowledge", query, k)
        enriched = []
        for r in results:
            meta = self.db.get("knowledge:meta", r["id"])
            enriched.append({
                **r,
                "metadata": json.loads(meta) if meta else {}
            })
        return enriched

    def get_context(self, query: str, k: int = 3) -> str:
        results = self.retrieve(query, k)
        context = "\n\n".join([
            f"[Source: {r['id']}]\n{r['content']}"
            for r in results
        ])
        return context

# Usage
rag = RAGSystem("./rag_db")

# Add documents
rag.add_document("wiki:1", "Python was created by Guido van Rossum...", {"source": "wikipedia"})
rag.add_document("wiki:2", "Machine learning is a subset of AI...", {"source": "wikipedia"})

# Retrieve context for a query
context = rag.get_context("Who created Python?")
print(context)
```

### Memory System

```python
from liath import Liath
import json
from datetime import datetime

class AgentMemory:
    def __init__(self, agent_id: str, data_dir: str):
        self.db = Liath(data_dir)
        self.agent_id = agent_id
        self.namespace = f"agent:{agent_id}:memory"
        self.db.create_namespace(self.namespace, 384)

    def remember(self, content: str, tags: list = None, importance: float = 0.5):
        mem_id = f"mem:{datetime.now().timestamp()}"
        self.db.store_with_embedding(self.namespace, mem_id, content)
        self.db.put(f"{self.namespace}:meta", mem_id, json.dumps({
            "tags": tags or [],
            "importance": importance,
            "timestamp": datetime.now().isoformat()
        }))
        return mem_id

    def recall(self, query: str, k: int = 5) -> list:
        results = self.db.semantic_search(self.namespace, query, k)
        memories = []
        for r in results:
            meta = json.loads(self.db.get(f"{self.namespace}:meta", r["id"]) or "{}")
            memories.append({
                "content": r["content"],
                "distance": r["distance"],
                **meta
            })
        return memories

    def recall_by_importance(self, query: str, min_importance: float = 0.5, k: int = 10) -> list:
        results = self.recall(query, k * 2)
        return [m for m in results if m.get("importance", 0) >= min_importance][:k]

# Usage
memory = AgentMemory("assistant", "./memory_db")

# Store memories
memory.remember("User prefers dark mode", ["preferences", "ui"], importance=0.8)
memory.remember("User works with Python", ["skills", "programming"], importance=0.9)
memory.remember("Discussed weather today", ["small-talk"], importance=0.2)

# Recall
relevant = memory.recall("What programming does the user know?", k=3)
for m in relevant:
    print(f"- {m['content']} (importance: {m.get('importance', 'N/A')})")
```

### Jupyter Notebook Usage

```python
# Cell 1: Setup
from liath import Liath
import pandas as pd
import json

db = Liath("./notebook_db")
db.create_namespace("research", 384)

# Cell 2: Load data
papers = pd.read_csv("papers.csv")

for _, row in papers.iterrows():
    db.store_with_embedding("research", f"paper:{row['id']}", row["abstract"])
    db.put("research:meta", f"paper:{row['id']}", json.dumps({
        "title": row["title"],
        "authors": row["authors"],
        "year": row["year"]
    }))

print(f"Loaded {len(papers)} papers")

# Cell 3: Search
query = "transformer neural networks"
results = db.semantic_search("research", query, k=10)

# Display as DataFrame
results_df = pd.DataFrame([
    {
        "id": r["id"],
        "distance": r["distance"],
        **json.loads(db.get("research:meta", r["id"]) or "{}")
    }
    for r in results
])
results_df
```

## Type Hints

For better IDE support, use type hints:

```python
from typing import List, Dict, Optional

def search_documents(
    db: "Liath",
    query: str,
    namespace: str = "docs",
    k: int = 10
) -> List[Dict[str, any]]:
    """Search for documents semantically."""
    return db.semantic_search(namespace, query, k)
```

## Error Handling

```python
from liath import Liath, LiathError

db = Liath("./data")

try:
    db.semantic_search("nonexistent", "query", 5)
except LiathError as e:
    print(f"Liath error: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

## See Also

- [Quick Start](../getting-started/quick-start.md) - Basic concepts
- [API Reference](../api/embedded-liath.md) - Full API details
- [Examples](../examples/index.md) - More code examples
