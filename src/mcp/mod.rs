//! MCP (Model Context Protocol) server for Liath
//!
//! Exposes Liath's database capabilities as MCP tools for AI assistants.

mod server;
mod tools;

pub use server::run_mcp_server;
