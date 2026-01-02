# MCP Server

Liath includes a Model Context Protocol (MCP) server for integration with AI assistants like Claude.

## Overview

MCP (Model Context Protocol) enables AI assistants to interact with external tools and data sources. Liath's MCP server exposes database operations as tools that AI assistants can use.

## Enabling MCP

Add the `mcp` feature to your `Cargo.toml`:

```toml
[dependencies]
liath = { version = "0.1", features = ["mcp"] }
```

## Starting the Server

### Via CLI

```bash
liath mcp
```

### Programmatically

```rust
use liath::{EmbeddedLiath, Config};
use liath::mcp::run_mcp_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;
    let executor = db.query_executor();

    run_mcp_server(executor, "default_user".to_string()).await?;

    Ok(())
}
```

## Claude Desktop Integration

### Configuration

Add to your Claude Desktop configuration (`claude_desktop_config.json`):

```json
{
    "mcpServers": {
        "liath": {
            "command": "liath",
            "args": ["mcp"],
            "env": {
                "LIATH_DATA_DIR": "/path/to/data"
            }
        }
    }
}
```

### Configuration Locations

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

## Available Tools

### Database Operations

#### liath_execute_lua

Execute Lua code for complex queries.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `code` | string | Lua code to execute |

**Example:**

```
Execute this Lua code:
local results = semantic_search("docs", "machine learning", 5)
return json.encode(results)
```

#### liath_kv_get

Retrieve a value by key.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `namespace` | string | Target namespace |
| `key` | string | Key to retrieve |

#### liath_kv_put

Store a key-value pair.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `namespace` | string | Target namespace |
| `key` | string | Key to store |
| `value` | string | Value to store |

#### liath_kv_delete

Delete a key-value pair.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `namespace` | string | Target namespace |
| `key` | string | Key to delete |

### Namespace Operations

#### liath_list_namespaces

List all available namespaces.

**Parameters:** None

#### liath_create_namespace

Create a new namespace.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `name` | string | Namespace name |
| `dimensions` | number | Vector dimensions |
| `metric` | string | Distance metric (cosine, euclidean) |

#### liath_delete_namespace

Delete a namespace.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `name` | string | Namespace to delete |

### Vector Operations

#### liath_semantic_search

Search for similar content.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `namespace` | string | Target namespace |
| `query` | string | Search query |
| `k` | number | Number of results |

#### liath_store_document

Store a document with embedding.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `namespace` | string | Target namespace |
| `id` | string | Document ID |
| `content` | string | Document content |

#### liath_generate_embeddings

Generate embeddings for texts.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `texts` | array | List of texts to embed |

### Agent Operations

#### liath_agent_create

Create a new agent.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `id` | string | Agent ID |
| `description` | string | Agent description |

#### liath_agent_list

List all registered agents.

**Parameters:** None

#### liath_agent_store_memory

Store a memory for an agent.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `agent_id` | string | Agent ID |
| `content` | string | Memory content |
| `tags` | array | Memory tags |

#### liath_agent_recall_memory

Recall memories semantically.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `agent_id` | string | Agent ID |
| `query` | string | Search query |
| `k` | number | Number of results |

#### liath_agent_recall_by_tags

Recall memories by tags.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `agent_id` | string | Agent ID |
| `tags` | array | Tags to filter |
| `limit` | number | Maximum results |

### Conversation Operations

#### liath_agent_create_conversation

Create a new conversation.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `agent_id` | string | Agent ID |
| `conversation_id` | string | Optional conversation ID |

#### liath_agent_load_conversation

Load an existing conversation.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `agent_id` | string | Agent ID |
| `conversation_id` | string | Conversation ID |

#### liath_agent_add_message

Add a message to a conversation.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `agent_id` | string | Agent ID |
| `conversation_id` | string | Conversation ID |
| `role` | string | Message role (user, assistant, system) |
| `content` | string | Message content |

#### liath_agent_get_messages

Get messages from a conversation.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `agent_id` | string | Agent ID |
| `conversation_id` | string | Conversation ID |
| `limit` | number | Maximum messages |

#### liath_agent_search_messages

Search messages in a conversation.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `agent_id` | string | Agent ID |
| `conversation_id` | string | Conversation ID |
| `query` | string | Search query |
| `k` | number | Number of results |

### Persistence

#### liath_save

Save all data to disk.

**Parameters:** None

## Usage Examples

### Storing Knowledge

```
User: Store this fact in my knowledge base: "The speed of light is 299,792,458 meters per second"

Claude: I'll store that fact using the Liath database.
[Uses liath_store_document with namespace="knowledge", content="The speed of light is 299,792,458 meters per second"]
```

### Semantic Search

```
User: What do I know about physics?

Claude: Let me search your knowledge base.
[Uses liath_semantic_search with namespace="knowledge", query="physics", k=5]
Found relevant entries about the speed of light and other physics concepts.
```

### Agent Memory

```
User: Remember that I prefer Python for data analysis

Claude: I'll store that preference in my memory.
[Uses liath_agent_store_memory with content="User prefers Python for data analysis", tags=["preferences", "programming"]]
```

### Complex Queries

```
User: Find my most important recent memories about work

Claude: I'll use a custom Lua query for that.
[Uses liath_execute_lua with code that searches memories, filters by importance, and ranks by recency]
```

## Protocol Details

### Transport

MCP uses JSON-RPC 2.0 over stdio (standard input/output).

### Message Format

**Request:**

```json
{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
        "name": "liath_semantic_search",
        "arguments": {
            "namespace": "docs",
            "query": "machine learning",
            "k": 5
        }
    }
}
```

**Response:**

```json
{
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
        "content": [
            {
                "type": "text",
                "text": "[{\"id\":\"doc:1\",\"content\":\"...\",\"distance\":0.123}]"
            }
        ]
    }
}
```

## Troubleshooting

### Server Not Starting

1. Check Liath is installed: `which liath`
2. Verify MCP feature: `liath --version`
3. Check config path is correct

### Tools Not Available

1. Restart Claude Desktop after config change
2. Check JSON syntax in config file
3. Verify server output: `liath mcp 2>&1 | head`

### Permission Errors

1. Check data directory permissions
2. Ensure write access to data path
3. Try different data directory

## See Also

- [HTTP Server](http-server.md) - REST API alternative
- [Agent API](../api/agent-api.md) - Agent operations
- [Lua Standard Library](../api/lua-stdlib.md) - Lua functions
