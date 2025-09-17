# Architecture

This document outlines Liath’s module layout and how the pieces fit together.

## Modules

- `core/`
  - `fjall_wrapper.rs`: Thin wrapper around Fjall `Keyspace`/`PartitionHandle` providing `put/get/delete` and batch operations.
  - `namespace.rs`: In‑memory registry of namespaces. Each namespace bundles a KV partition and (optionally) a vector index.

- `vector/` (feature: `vector`)
  - `usearch_wrapper.rs`: Wraps a USearch `Index` per namespace with `add/search/save/load` helpers.

- `ai/` (feature: `embedding`)
  - `embedding.rs`: Wrapper around FastEmbed `TextEmbedding` with simple `generate()` helpers.

- `lua/`
  - `vm.rs`: Lua VM wrapper (rlua). Exposes `execute` helpers.
  - `luarocks.rs`: Optional LuaRocks interactions (invokes external `luarocks` if present).

- `file/`
  - `storage.rs`: Content‑addressed file storage using the filesystem.
  - `processing.rs`: Placeholder helpers for text/image extraction.

- `auth/`
  - `manager.rs`: Simple in‑memory user → permissions mapping.

- `query/`
  - `executor.rs`: Registers functions into Lua, provides the async `execute(query, user_id)` entrypoint, and exposes typed Rust helpers (create_namespace, put/get/delete, etc.).
  - `parser.rs`: Minimal string parser for future non‑Lua query support.

- `cli/`
  - `console.rs`: Interactive console with a few typed commands; falls back to Lua.

- `server/` (feature: `server`)
  - `api.rs`: Axum HTTP API. Uses a `LocalSet` worker that owns `QueryExecutor`. Handlers send requests over `mpsc` and receive responses via `oneshot`.

## Data Flow

1. KV paths
   - Typed APIs (`QueryExecutor::put/get/delete`) → `NamespaceManager` → `FjallWrapper` → Fjall partition.

2. Vector paths (optional)
   - `create_namespace` constructs a USearch index for that namespace.
   - `similarity_search` queries the index and returns `(id, distance)` pairs.

3. Embedding paths (optional)
   - `EmbeddingWrapper` (FastEmbed) generates vectors from text.
   - Combine with vector search to build semantic retrieval.

4. Lua execution
   - `QueryExecutor::execute` registers functions (namespace ops, kv ops, embeddings, file ops, packages) into the Lua context and evaluates the query.
   - Results are normalized to strings for HTTP/CLI responses.

## Concurrency

- Shared components (`NamespaceManager`, `EmbeddingWrapper`, `FileStorage`, `AuthManager`) live behind `Arc<RwLock<...>>`.
- Embedding operations use a `Semaphore` to cap concurrent work.
- Server uses a single‑threaded worker (`LocalSet`) that owns `QueryExecutor` to avoid `Send/Sync` constraints of the Lua VM. Axum handlers run on the main runtime and communicate via channels.

## Configuration

- `Config { data_dir, luarocks_path }` initializes storage and optional LuaRocks path.
- Feature flags toggle optional subsystems:
  - `embedding` → FastEmbed / ONNX Runtime
  - `vector` → USearch
  - `server` → Axum HTTP server

## Layout

```
src/
  ai/          # embeddings (optional)
  auth/        # simple auth
  cli/         # interactive console
  core/        # fjall wrapper + namespaces
  file/        # file storage
  lua/         # lua vm + luarocks helper
  query/       # executor + parser
  server/      # http api (optional)
```
