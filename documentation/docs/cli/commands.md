# CLI Commands

Complete reference for all Liath CLI commands.

## Global Options

These options apply to all commands:

```bash
liath [OPTIONS] <COMMAND>

OPTIONS:
    -d, --data-dir <PATH>    Data directory [default: ./liath_data]
    -h, --help               Print help information
    -V, --version            Print version
```

## tui

Start the interactive Terminal User Interface.

```bash
liath tui [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--data-dir <PATH>` | Data directory |

**Example:**

```bash
liath tui
liath tui --data-dir /var/lib/liath
```

## cli

Start the command-line REPL.

```bash
liath cli [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--data-dir <PATH>` | Data directory |

**Example:**

```bash
liath cli
```

**REPL Commands:**

```
liath> :help
Available commands:
  :ns list                              - List all namespaces
  :ns create <name> <dims> <metric> <type> - Create namespace
  :put <ns> <key> <value>               - Store value
  :get <ns> <key>                       - Retrieve value
  :del <ns> <key>                       - Delete value
  :quit                                 - Exit

Or enter Lua code directly:

liath> return 1 + 2
3

liath> store_with_embedding("docs", "d1", "Hello world")
nil

liath> return json.encode(semantic_search("docs", "greeting", 5))
[{"id":"d1","content":"Hello world","distance":0.123}]
```

## server

Start the HTTP API server.

```bash
liath server [OPTIONS]
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--host <HOST>` | Bind address | `127.0.0.1` |
| `--port <PORT>` | Port number | `8080` |
| `--data-dir <PATH>` | Data directory | `./liath_data` |

**Example:**

```bash
# Default (localhost:8080)
liath server

# Custom host and port
liath server --host 0.0.0.0 --port 3000

# With data directory
liath server --data-dir /var/lib/liath --host 0.0.0.0
```

**Output:**

```
Starting Liath HTTP server...
Listening on http://0.0.0.0:8080
```

## mcp

Start the Model Context Protocol server.

```bash
liath mcp [OPTIONS]
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--data-dir <PATH>` | Data directory | `./liath_data` |
| `--user <USER_ID>` | Default user ID | `mcp_user` |

**Example:**

```bash
liath mcp
liath mcp --data-dir /var/lib/liath
```

**Note:** MCP uses stdio for communication. Configure in your AI assistant's settings.

## execute

Execute Lua code directly.

```bash
liath execute <CODE> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<CODE>` | Lua code to execute |

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--data-dir <PATH>` | Data directory | `./liath_data` |
| `--user <USER_ID>` | User ID for permissions | `cli_user` |
| `--json` | Output as JSON | false |

**Examples:**

```bash
# Simple calculation
liath execute 'return 1 + 2'
# Output: 3

# JSON output
liath execute 'return json.encode({a = 1, b = 2})' --json
# Output: {"a":1,"b":2}

# Multi-line (use quotes)
liath execute '
local x = 10
local y = 20
return x + y
'
# Output: 30

# Database operations
liath execute 'put("test", "key", "value"); return get("test", "key")'
# Output: value

# Semantic search
liath execute '
store_with_embedding("docs", "d1", "Hello world")
local results = semantic_search("docs", "greeting", 5)
return json.encode(results)
'
```

## namespace

Manage namespaces.

```bash
liath namespace <SUBCOMMAND>
```

### namespace list

List all namespaces.

```bash
liath namespace list [OPTIONS]
```

**Example:**

```bash
liath namespace list
# Output:
# Namespaces:
#   - default
#   - documents
#   - memories
```

### namespace create

Create a new namespace.

```bash
liath namespace create <NAME> <DIMENSIONS> <METRIC> <SCALAR>
```

**Arguments:**

| Argument | Description | Values |
|----------|-------------|--------|
| `<NAME>` | Namespace name | Any string |
| `<DIMENSIONS>` | Vector dimensions | 384, 768, etc. |
| `<METRIC>` | Distance metric | `cosine`, `euclidean`, `ip` |
| `<SCALAR>` | Scalar type | `f32`, `f16`, `i8` |

**Example:**

```bash
# Standard text embedding namespace
liath namespace create documents 384 cosine f32

# Memory-efficient namespace
liath namespace create large_corpus 384 cosine f16

# Image features namespace
liath namespace create images 512 euclidean f32
```

### namespace delete

Delete a namespace.

```bash
liath namespace delete <NAME>
```

**Example:**

```bash
liath namespace delete old_namespace
```

!!! danger "Warning"
    Deleting a namespace permanently removes all data. This cannot be undone.

### namespace info

Show namespace information.

```bash
liath namespace info <NAME>
```

**Example:**

```bash
liath namespace info documents
# Output:
# Namespace: documents
#   Dimensions: 384
#   Metric: cosine
#   Scalar: f32
#   Vectors: 1234
```

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Configuration error |
| 4 | Runtime error |

## Shell Completion

Generate shell completions:

```bash
# Bash
liath completions bash > /etc/bash_completion.d/liath

# Zsh
liath completions zsh > ~/.zsh/completions/_liath

# Fish
liath completions fish > ~/.config/fish/completions/liath.fish
```

## Scripting Examples

### Batch Import

```bash
#!/bin/bash

# Create namespace
liath namespace create docs 384 cosine f32

# Import documents
for file in ./documents/*.txt; do
    id=$(basename "$file" .txt)
    content=$(cat "$file")
    liath execute "store_with_embedding('docs', '$id', [[$content]])"
done

echo "Import complete"
```

### Backup

```bash
#!/bin/bash

# Export all namespaces
liath namespace list | while read ns; do
    if [ "$ns" != "Namespaces:" ] && [ -n "$ns" ]; then
        ns_name=$(echo "$ns" | sed 's/^  - //')
        echo "Backing up $ns_name..."
        liath execute "
            local keys = keys('$ns_name')
            local data = {}
            for _, k in ipairs(keys) do
                data[k] = get('$ns_name', k)
            end
            return json.encode(data)
        " > "backup_${ns_name}.json"
    fi
done
```

### Health Check

```bash
#!/bin/bash

# Check if server is healthy
response=$(curl -s http://localhost:8080/health)
status=$(echo "$response" | jq -r '.status')

if [ "$status" = "healthy" ]; then
    echo "Server is healthy"
    exit 0
else
    echo "Server is unhealthy"
    exit 1
fi
```

## See Also

- [Interactive Mode](interactive.md) - REPL details
- [TUI](tui.md) - Terminal UI guide
- [HTTP Server](../integrations/http-server.md) - Server API
