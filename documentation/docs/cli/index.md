# CLI Reference

Liath provides a command-line interface for database operations, interactive exploration, and server management.

## Installation

The CLI is included when you install Liath:

```bash
cargo install liath
```

Or build from source:

```bash
cargo build --release
./target/release/liath --help
```

## Commands Overview

```bash
liath [COMMAND] [OPTIONS]

COMMANDS:
    tui       Start interactive TUI (default)
    cli       Start command-line REPL
    server    Start HTTP API server
    mcp       Start MCP server
    execute   Execute Lua code
    namespace Manage namespaces
    help      Print help information

OPTIONS:
    -d, --data-dir <PATH>    Data directory [default: ./liath_data]
    -h, --help               Print help
    -V, --version            Print version
```

## Quick Navigation

<div class="grid cards" markdown>

-   :material-console:{ .lg .middle } **Commands**

    ---

    All CLI commands and options

    [:octicons-arrow-right-24: Commands](commands.md)

-   :material-console-line:{ .lg .middle } **Interactive Mode**

    ---

    REPL for direct queries

    [:octicons-arrow-right-24: Interactive](interactive.md)

-   :material-monitor:{ .lg .middle } **TUI**

    ---

    Terminal user interface

    [:octicons-arrow-right-24: TUI](tui.md)

</div>

## Quick Examples

### Start TUI

```bash
liath
# or
liath tui
```

### Start CLI REPL

```bash
liath cli
```

### Execute Lua Code

```bash
liath execute 'return 1 + 2 + 3'
# Output: 6

liath execute 'return json.encode({hello = "world"})'
# Output: {"hello":"world"}
```

### Start HTTP Server

```bash
liath server --host 0.0.0.0 --port 8080
```

### Manage Namespaces

```bash
# List namespaces
liath namespace list

# Create namespace
liath namespace create documents 384 cosine f32

# Delete namespace
liath namespace delete old_namespace
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `LIATH_DATA_DIR` | Data directory path | `./liath_data` |
| `LIATH_LOG_LEVEL` | Log level (trace, debug, info, warn, error) | `info` |
| `LIATH_HOST` | Server bind host | `127.0.0.1` |
| `LIATH_PORT` | Server bind port | `8080` |

### Data Directory

```bash
# Set via flag
liath --data-dir /var/lib/liath tui

# Set via environment
export LIATH_DATA_DIR=/var/lib/liath
liath tui
```

## See Also

- [Commands](commands.md) - Detailed command reference
- [Interactive Mode](interactive.md) - REPL guide
- [TUI](tui.md) - Terminal UI guide
