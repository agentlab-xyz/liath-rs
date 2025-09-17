# TODO / Issues

## Compilation

### 1. Fastembed API Issues
- [x] Fix `dim()` method calls - should use `get_metadata().dimension`
- [x] Fix `InitOptions` construction - should use `InitOptions::new()`
- [x] Fix `embed()` method calls - ensure correct signature

### 2. Lua VM Issues
- [x] Fix `context()` method calls - need to import `RluaCompat` trait
- [ ] Fix moved value errors in QueryExecutor

### 3. QueryExecutor Issues
- [ ] Fix moved value errors for `user_id`, `auth_manager`, `namespace_manager`, `file_storage`, `lua_vm`
- [x] Remove LLM dependencies since we're not using them currently
- [ ] Fix `anyhow::Error` trait bound issue

### 4. Additional Issues
- [ ] Fix `get_metadata` method not found for `TextEmbedding`
- [ ] Fix borrowing data in an `Arc` as mutable
- [ ] Fix unused imports and variables
- [ ] Fix deprecated method usage

## Features

### 1. Core Database Functionality
- [ ] Implement RocksDB wrapper functionality
- [ ] Implement Usearch wrapper functionality
- [ ] Implement namespace management
- [ ] Implement authentication and authorization

### 2. Embedding Functionality
- [ ] Test embedding generation with fastembed
- [ ] Implement vector search functionality
- [ ] Add support for different embedding models

### 3. Lua Integration
- [ ] Implement Lua VM functionality
- [ ] Register database functions in Lua context
- [ ] Implement file storage operations in Lua

### 4. CLI and Server
- [ ] Implement CLI interface
- [ ] Implement HTTP server with Axum
- [ ] Add API endpoints for database operations

### 5. Documentation
- [ ] Update README with usage examples
- [ ] Document API endpoints
- [ ] Add examples for embedding generation
