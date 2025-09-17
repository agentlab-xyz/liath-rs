use clap::{Parser, Subcommand};
use liath::{EmbeddedLiath, Config};
use anyhow::Result;
use liath::cli::console;
#[cfg(feature = "server")]
use liath::server::run_server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Cli,
    Server { port: Option<u16> },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Create a minimal config without device-specific settings
    let config = Config::default();

    let liath = EmbeddedLiath::new(config)?;

    match &cli.command {
        Some(Commands::Cli) => {
            console::run(liath.query_executor()).await?;
        }
        Some(Commands::Server { port }) => {
            let port = port.unwrap_or(3000);
            #[cfg(feature = "server")]
            {
                run_server(port, liath.query_executor()).await?;
            }
            #[cfg(not(feature = "server"))]
            {
                println!("Rebuild with `--features server` to enable HTTP server.");
            }
        }
        None => {
            println!("Please specify a command: cli or server");
        }
    }

    Ok(())
}
