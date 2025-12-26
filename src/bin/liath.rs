//! Liath - AI-First Database
//!
//! A high-performance database with built-in AI capabilities including
//! vector search, embeddings, and a Lua scripting interface.

use clap::{Parser, Subcommand, Args};
use liath::{EmbeddedLiath, Config};
use anyhow::Result;
use std::path::PathBuf;

#[cfg(feature = "server")]
use liath::server::run_server;

/// Liath - AI-First Database with vector search and Lua scripting
#[derive(Parser)]
#[command(
    name = "liath",
    author,
    version,
    about = "AI-First Database with vector search, embeddings, and Lua scripting",
    long_about = r#"
Liath is a high-performance database designed for AI applications.

Features:
  - Key-value storage with persistence
  - Vector similarity search (cosine, euclidean)
  - Built-in embedding generation
  - Lua scripting interface
  - Agent memory and conversation management

Examples:
  liath                     Start interactive TUI console
  liath cli                 Start interactive console
  liath cli --simple        Start simple readline console
  liath server              Start HTTP API server on port 3000
  liath server --port 8080  Start server on custom port
  liath mcp                 Start MCP server (for AI assistants)
  liath execute "print('hello')"  Execute a Lua script
"#
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Data directory for persistence
    #[arg(short, long, global = true, default_value = "./data")]
    data_dir: PathBuf,

    /// User ID for authentication
    #[arg(short, long, global = true, default_value = "admin")]
    user: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the interactive console (default)
    #[command(alias = "console", alias = "repl")]
    Cli(CliArgs),

    /// Start the HTTP API server
    #[command(alias = "serve")]
    Server(ServerArgs),

    /// Execute a Lua script and exit
    #[command(alias = "exec", alias = "run")]
    Execute(ExecuteArgs),

    /// Manage namespaces
    #[command(alias = "ns")]
    Namespace(NamespaceArgs),

    /// Start MCP server for AI assistant integration
    Mcp,

    /// Display version and build information
    Info,
}

#[derive(Args)]
struct CliArgs {
    /// Use simple readline interface instead of TUI
    #[arg(short, long)]
    simple: bool,
}

#[derive(Args)]
struct ServerArgs {
    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Host to bind to
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,
}

#[derive(Args)]
struct ExecuteArgs {
    /// Lua code to execute
    code: String,

    /// Execute from file instead of command line
    #[arg(short, long)]
    file: Option<PathBuf>,
}

#[derive(Args)]
struct NamespaceArgs {
    #[command(subcommand)]
    action: NamespaceAction,
}

#[derive(Subcommand)]
enum NamespaceAction {
    /// List all namespaces
    List,

    /// Create a new namespace
    Create {
        /// Name of the namespace
        name: String,

        /// Vector dimensions (default: 384 for all-MiniLM-L6-v2)
        #[arg(short, long, default_value = "384")]
        dimensions: usize,

        /// Distance metric: cosine or euclidean
        #[arg(short, long, default_value = "cosine")]
        metric: String,
    },

    /// Delete a namespace
    Delete {
        /// Name of the namespace to delete
        name: String,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging (only in debug mode or when RUST_LOG is set)
    if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::fmt::init();
    }

    let cli = Cli::parse();

    // Handle commands that don't need full initialization first
    if let Some(Commands::Info) = &cli.command {
        println!("Liath - AI-First Database");
        println!("Version: {}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("Features:");
        #[cfg(feature = "embedding")]
        println!("  - Embedding generation (fastembed)");
        #[cfg(not(feature = "embedding"))]
        println!("  - Embedding generation: disabled");
        #[cfg(feature = "vector")]
        println!("  - Vector search (usearch)");
        #[cfg(not(feature = "vector"))]
        println!("  - Vector search: disabled");
        #[cfg(feature = "server")]
        println!("  - HTTP server (axum)");
        #[cfg(not(feature = "server"))]
        println!("  - HTTP server: disabled");
        #[cfg(feature = "tui")]
        println!("  - TUI console (ratatui)");
        #[cfg(not(feature = "tui"))]
        println!("  - TUI console: disabled");
        println!();
        println!("Data directory: {}", cli.data_dir.display());
        return Ok(());
    }

    // Create config with data directory
    let config = Config {
        data_dir: cli.data_dir.clone(),
        ..Default::default()
    };

    let liath = EmbeddedLiath::new(config)?;
    let query_executor = liath.query_executor();

    match cli.command {
        // Default: start TUI console
        None => {
            #[cfg(feature = "tui")]
            {
                liath::cli::tui::run(query_executor, cli.user, cli.data_dir).await?;
            }
            #[cfg(not(feature = "tui"))]
            {
                liath::cli::console::run(query_executor).await?;
            }
        }

        Some(Commands::Cli(args)) => {
            if args.simple {
                liath::cli::console::run(query_executor).await?;
            } else {
                #[cfg(feature = "tui")]
                {
                    liath::cli::tui::run(query_executor, cli.user, cli.data_dir).await?;
                }
                #[cfg(not(feature = "tui"))]
                {
                    liath::cli::console::run(query_executor).await?;
                }
            }
        }

        Some(Commands::Server(args)) => {
            #[cfg(feature = "server")]
            {
                println!("Starting Liath server on {}:{}", args.host, args.port);
                run_server(args.port, query_executor).await?;
            }
            #[cfg(not(feature = "server"))]
            {
                let _ = args;
                eprintln!("Error: Server feature not enabled.");
                eprintln!("Rebuild with: cargo build --features server");
                std::process::exit(1);
            }
        }

        Some(Commands::Execute(args)) => {
            let code = if let Some(file) = args.file {
                std::fs::read_to_string(&file)?
            } else {
                args.code
            };

            match query_executor.execute(&code, &cli.user).await {
                Ok(result) => {
                    if !result.is_empty() {
                        println!("{}", result);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Namespace(ns_args)) => {
            match ns_args.action {
                NamespaceAction::List => {
                    let namespaces = query_executor.list_namespaces();
                    if namespaces.is_empty() {
                        println!("No namespaces found.");
                    } else {
                        println!("Namespaces:");
                        for ns in namespaces {
                            println!("  - {}", ns);
                        }
                    }
                }

                NamespaceAction::Create { name, dimensions, metric } => {
                    #[cfg(feature = "vector")]
                    {
                        use usearch::{MetricKind, ScalarKind};
                        let metric_kind = match metric.to_lowercase().as_str() {
                            "euclidean" | "l2" => MetricKind::L2sq,
                            _ => MetricKind::Cos,
                        };
                        match query_executor.create_namespace(&name, dimensions, metric_kind, ScalarKind::F32) {
                            Ok(_) => println!("Created namespace '{}' ({}D, {})", name, dimensions, metric),
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    #[cfg(not(feature = "vector"))]
                    {
                        let _ = (dimensions, metric);
                        match query_executor.create_namespace_basic(&name) {
                            Ok(_) => println!("Created namespace '{}'", name),
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                }

                NamespaceAction::Delete { name, force } => {
                    if !force {
                        println!("Are you sure you want to delete namespace '{}'? [y/N]", name);
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                        if !input.trim().eq_ignore_ascii_case("y") {
                            println!("Aborted.");
                            return Ok(());
                        }
                    }
                    match query_executor.delete_namespace(&name) {
                        Ok(_) => println!("Deleted namespace '{}'", name),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }

        Some(Commands::Mcp) => {
            #[cfg(feature = "mcp")]
            {
                eprintln!("Starting Liath MCP server...");
                liath::mcp::run_mcp_server(query_executor, cli.user).await?;
            }
            #[cfg(not(feature = "mcp"))]
            {
                eprintln!("Error: MCP feature not enabled.");
                eprintln!("Rebuild with: cargo build --features mcp");
                std::process::exit(1);
            }
        }

        Some(Commands::Info) => {
            // Handled early, before initialization
            unreachable!()
        }
    }

    Ok(())
}
