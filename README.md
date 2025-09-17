# Liath

Liath is a small, embeddable data engine with:
- Key–value storage built on Fjall
- Optional vector search (USearch)
- Optional text embeddings (FastEmbed / ONNX Runtime)
- A Lua runtime for scripting
- A CLI and an optional HTTP API

Liath emphasizes a simple, composable core you can embed in your own projects, with opt‑in features when you need search, embeddings, or an HTTP interface.

## Features

- Storage: Namespaced key–value store using Fjall
- Scripting: Execute Lua to interact with data and utilities
- Vector search: USearch index per namespace (optional)
- Embeddings: FastEmbed models for text (optional)
- HTTP API: Axum server (optional)

Feature flags (Cargo):
- `embedding` (default): enable FastEmbed/ONNX Runtime
- `vector` (default): enable USearch
- `server` (off by default): enable Axum HTTP API

## Quick Start

Prerequisites:
- Rust (stable)
- System requirements listed in docs/system-deps.md

Build from source:
```bash
git clone https://github.com/nudgelang/liath-rs.git
cd liath-rs
cargo build
```

Run the CLI:
```bash
cargo run --bin liath -- cli
```

Start the HTTP server (localhost:3000):
```bash
cargo run --features server --bin liath -- server --port 3000
```

Use as a library (typed API):
```rust
use liath::{EmbeddedLiath, Config};

fn main() -> anyhow::Result<()> {
    let liath = EmbeddedLiath::new(Config::default())?;

    // Create a namespace, then put/get a key
    liath.create_namespace("docs", 128, usearch::MetricKind::Cos, usearch::ScalarKind::F32)?;
    liath.put("docs", b"hello", b"world")?;
    let value = liath.get("docs", b"hello")?;
    assert_eq!(value.as_deref(), Some(b"world".as_ref()));
    Ok(())
}
```

## Documentation

Start here:
- docs/guide.md — quickstart, CLI and server usage
- docs/system-deps.md — platform packages
- docs/summary.md — current state and roadmap

The code is organized into small modules under `src/` (core, ai, vector, lua, file, query, auth, cli, server).

## Status

Liath is under active development. The storage core and CLI are usable; server and embeddings are opt‑in and evolving. See docs/status.md and docs/tasks.md for details.

## License

MIT
