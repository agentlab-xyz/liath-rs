# Guide

This guide covers the basics: building, running the CLI, and starting the HTTP server.

## Build

```bash
cargo build
```

Feature flags:
- `embedding` (default) — FastEmbed / ONNX Runtime
- `vector` (default) — USearch
- `server` — Axum HTTP API

Disable optional defaults if desired:
```bash
cargo build --no-default-features
```

## CLI

Start the console:
```bash
cargo run --bin liath -- cli
```

Helper commands:
- `:ns list`
- `:ns create <name> <dims> <cosine|euclidean> <f32|f16>`
- `:put <namespace> <key> <value...>`
- `:get <namespace> <key>`
- `:del <namespace> <key>`

Any other input is executed as Lua.

## HTTP Server

Enable the `server` feature and run:
```bash
cargo run --features server --bin liath -- server --port 3000
```

POST a query:
```bash
curl -s localhost:3000/query \
  -H 'content-type: application/json' \
  -d '{"query":"return 1+1","user_id":"admin"}'
```

The server executes queries on a single-threaded worker that owns the Lua VM and data engine.

