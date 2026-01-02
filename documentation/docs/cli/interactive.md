# Interactive Mode

The CLI REPL provides an interactive environment for Liath operations.

## Starting Interactive Mode

```bash
liath cli
```

## REPL Interface

```
Welcome to Liath CLI
Type :help for available commands, or enter Lua code directly.

liath>
```

## Built-in Commands

All built-in commands start with `:`.

### :help

Show available commands:

```
liath> :help
Available commands:
  :ns list                              - List all namespaces
  :ns create <name> <dims> <metric> <type> - Create namespace
  :put <ns> <key> <value>               - Store value
  :get <ns> <key>                       - Retrieve value
  :del <ns> <key>                       - Delete value
  :search <ns> <query> [k]              - Semantic search
  :history                              - Show command history
  :clear                                - Clear screen
  :quit                                 - Exit

Enter Lua code directly for execution.
```

### :ns list

List all namespaces:

```
liath> :ns list
Namespaces:
  - default (384 dims, cosine)
  - documents (384 dims, cosine)
  - memories (384 dims, cosine)
```

### :ns create

Create a new namespace:

```
liath> :ns create mydata 384 cosine f32
Created namespace 'mydata' (384 dimensions, cosine metric, f32 scalar)
```

### :put

Store a value:

```
liath> :put mydata user:1 {"name": "Alice"}
Stored key 'user:1' in namespace 'mydata'
```

### :get

Retrieve a value:

```
liath> :get mydata user:1
{"name": "Alice"}
```

### :del

Delete a value:

```
liath> :del mydata user:1
Deleted key 'user:1' from namespace 'mydata'
```

### :search

Perform semantic search:

```
liath> :search documents machine learning 5
Results for 'machine learning' in 'documents':
  1. [doc:1] Introduction to neural networks... (dist: 0.123)
  2. [doc:2] Deep learning fundamentals... (dist: 0.234)
  3. [doc:3] ML model training... (dist: 0.345)
```

### :history

Show command history:

```
liath> :history
  1. :ns list
  2. :put test key1 value1
  3. return 1 + 2
  4. :search docs query 5
```

### :clear

Clear the screen.

### :quit

Exit the REPL (also: `:exit`, `:q`, Ctrl+D).

## Lua Execution

Enter Lua code directly:

```
liath> return 1 + 2 + 3
6

liath> return "Hello, " .. "World!"
Hello, World!

liath> return json.encode({a = 1, b = 2})
{"a":1,"b":2}
```

### Multi-line Input

Use backslash for continuation:

```
liath> local x = 10 \
... local y = 20 \
... return x + y
30
```

Or use `[[` for multi-line strings:

```
liath> return [[
... Line 1
... Line 2
... Line 3
... ]]
Line 1
Line 2
Line 3
```

### Database Operations

```
liath> put("test", "greeting", "Hello!")
nil

liath> return get("test", "greeting")
Hello!

liath> store_with_embedding("docs", "d1", "Introduction to Rust programming")
nil

liath> return json.encode(semantic_search("docs", "programming", 3))
[{"id":"d1","content":"Introduction to Rust programming","distance":0.123}]
```

### Complex Queries

```
liath> local function smart_search(query)
...   local results = semantic_search("docs", query, 10)
...   local filtered = filter(results, function(r) return r.distance < 0.5 end)
...   return filtered
... end
... return json.encode(smart_search("machine learning"))
[{"id":"d1","content":"...","distance":0.234}]
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+C` | Cancel current input |
| `Ctrl+D` | Exit REPL |
| `Ctrl+L` | Clear screen |
| `Up/Down` | Navigate history |
| `Tab` | Auto-complete (if available) |
| `Ctrl+A` | Move to line start |
| `Ctrl+E` | Move to line end |
| `Ctrl+W` | Delete word backward |
| `Ctrl+U` | Delete to line start |

## Output Formatting

### JSON Output

Results are displayed as-is. For pretty printing:

```
liath> local result = semantic_search("docs", "query", 5)
... -- Pretty print
... local json_str = json.encode(result)
... return json_str
```

### Table Display

For tabular data:

```
liath> local results = semantic_search("docs", "query", 5)
... for i, r in ipairs(results) do
...   print(string.format("%d. %s (%.3f)", i, r.id, r.distance))
... end
1. doc:1 (0.123)
2. doc:2 (0.234)
3. doc:3 (0.345)
```

## Configuration

### History File

Command history is saved to `~/.liath_history`.

### Prompt Customization

Set environment variable:

```bash
export LIATH_PROMPT="db> "
liath cli
```

## Examples

### Document Management

```
liath> :ns create docs 384 cosine f32
Created namespace 'docs'

liath> store_with_embedding("docs", "rust", "Rust is a systems programming language")
nil

liath> store_with_embedding("docs", "python", "Python is great for data science")
nil

liath> :search docs programming 2
Results:
  1. [rust] Rust is a systems programming language (dist: 0.123)
  2. [python] Python is great for data science (dist: 0.456)
```

### Memory Operations

```
liath> store_memory("agent:memory", "User prefers dark mode", {"preferences", "ui"})
nil

liath> return json.encode(recall("agent:memory", "UI preferences", 3))
[{"id":"...","content":"User prefers dark mode","distance":0.123}]
```

### Conversation

```
liath> add_message("conv:1", "user", "Hello!")
nil

liath> add_message("conv:1", "assistant", "Hi! How can I help?")
nil

liath> return json.encode(get_messages("conv:1", 10))
[{"role":"user","content":"Hello!"},{"role":"assistant","content":"Hi! How can I help?"}]
```

## Troubleshooting

### "Command not found"

Ensure you're using `:` prefix for built-in commands:

```
liath> ns list     -- Wrong
liath> :ns list    -- Correct
```

### "Syntax error"

Check Lua syntax. Common issues:

```lua
-- Wrong: missing quotes
return hello

-- Correct
return "hello"

-- Wrong: using = instead of ==
if x = 5 then

-- Correct
if x == 5 then
```

### "Namespace not found"

Create the namespace first:

```
liath> :ns create myns 384 cosine f32
liath> :put myns key value
```

## See Also

- [Commands](commands.md) - CLI command reference
- [TUI](tui.md) - Terminal UI
- [Lua Scripting](../guides/lua-scripting.md) - Lua guide
