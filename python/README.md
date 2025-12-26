# Liath

**Programmable Memory for AI Agents**

Liath enables agents to write Lua code to safely query their own memory. Instead of using fixed APIs, agents generate code that runs in a sandboxed environment with structured error feedback.

## Installation

```bash
pip install liath
```

## Quick Start

```python
from liath import Liath

# Initialize
db = Liath("./agent_data")

# Agent writes Lua to query memory
result = db.execute('''
    local results = search("memories", "coding preferences", 10)
    return json.encode(results)
''')

if result["success"]:
    memories = result["value"]
else:
    # Structured error helps LLM fix the code
    print(f"Error: {result['error']['message']}")
    print(f"Fix: {result['error']['suggestion']}")
```

## Key Features

### Safe Lua Execution
Agents write Lua code that runs in a sandbox with no file or network access:

```python
# Forbidden operations are blocked with helpful suggestions
result = db.execute("io.open('/etc/passwd')")
# result["error"]["suggestion"] = "Use put()/get() for data storage"
```

### Semantic Memory
Store and search content by meaning:

```python
# Store with automatic embedding
db.store("memories", "fact1", "User prefers dark mode")
db.store("memories", "fact2", "User codes in Python")

# Search by meaning
results = db.search("memories", "UI preferences", limit=5)
for r in results:
    print(f"{r.content} (distance: {r.distance})")
```

### Key-Value Storage
Simple persistent storage:

```python
db.put("settings", "theme", "dark")
theme = db.get("settings", "theme")
db.delete("settings", "theme")
```

### Validation Before Execution
Check Lua code without running it:

```python
validation = db.validate("return 1 + ")  # Syntax error
if not validation.valid:
    for error in validation.errors:
        print(f"Line {error.line}: {error.message}")
        print(f"Fix: {error.suggestion}")
```

## API Reference

### Liath Class

```python
class Liath:
    def __init__(self, data_dir: str = "./data") -> None: ...

    # Core operation - run agent-generated Lua
    def execute(self, code: str, user_id: str = "default") -> dict: ...

    # Validation without execution
    def validate(self, code: str) -> ValidationResult: ...

    # Semantic memory
    def store(self, namespace: str, key: str, content: str) -> None: ...
    def search(self, namespace: str, query: str, limit: int = 10) -> list[SearchResult]: ...

    # Key-value operations
    def put(self, namespace: str, key: str, value: str) -> None: ...
    def get(self, namespace: str, key: str) -> str | None: ...
    def delete(self, namespace: str, key: str) -> None: ...

    # Namespace management
    def create_namespace(self, name: str, dimensions: int = 384, metric: str = "cosine") -> None: ...
    def list_namespaces(self) -> list[str]: ...

    # Embedding generation
    def embed(self, text: str) -> list[float]: ...

    # Help
    def help(self) -> str: ...  # Available Lua functions

    # Lifecycle
    def save(self) -> None: ...
    def close(self) -> None: ...
```

### Available Lua Functions

```lua
-- Storage
put(namespace, key, value)
get(namespace, key) -> string|nil
delete(namespace, key)

-- Semantic Search
store(namespace, id, content)   -- auto-embeds
search(namespace, query, limit) -> [{id, content, distance}]

-- Utilities
json.encode(table) -> string
json.decode(string) -> table
now() -> timestamp
```

## Example: Agent Memory Loop

```python
from liath import Liath
from anthropic import Anthropic

db = Liath("./agent_data")
llm = Anthropic()

def query_memory(question: str) -> str:
    """Let the LLM write Lua to query memory."""

    # Ask LLM to generate query code
    prompt = f'''Generate Lua code to find memories related to: "{question}"

Available:
- search(namespace, query, limit) -> list
- json.encode(value) -> string

Return only Lua code.'''

    lua_code = llm.messages.create(
        model="claude-sonnet-4-20250514",
        messages=[{"role": "user", "content": prompt}]
    ).content[0].text

    # Execute with error handling
    result = db.execute(lua_code)

    if result["success"]:
        return result["value"]

    # Give LLM the error to fix
    fix_prompt = f'''Your Lua had an error:
{result["error"]["message"]}
Suggestion: {result["error"]["suggestion"]}

Fix this code:
{lua_code}'''

    fixed_code = llm.messages.create(
        model="claude-sonnet-4-20250514",
        messages=[{"role": "user", "content": fix_prompt}]
    ).content[0].text

    return db.execute(fixed_code)["value"]
```

## License

MIT
