use clap::{Parser, Subcommand};
use liath::{EmbeddedLiath, Config};
use anyhow::Result;
use candle_core::Device;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long, default_value = "cpu")]
    device: String,

    #[arg(long)]
    model_path: PathBuf,

    #[arg(long)]
    tokenizer_path: PathBuf,
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

    let device = match cli.device.as_str() {
        "cpu" => Device::Cpu,
        "cuda" => Device::new_cuda(0)?,
        _ => anyhow::bail!("Invalid device specified"),
    };

    let config = Config {
        device,
        model_path: cli.model_path,
        tokenizer_path: cli.tokenizer_path,
        data_dir: PathBuf::from("./data"), // TODO: Make configurable
    };

    let liath = EmbeddedLiath::new(config)?;

    match &cli.command {
        Some(Commands::Cli) => {
            // TODO: Implement CLI interface
            println!("CLI mode not yet implemented");
        }
        Some(Commands::Server { port }) => {
            let port = port.unwrap_or(3000);
            // TODO: Implement server interface
            println!("Server mode not yet implemented. Would run on port {}", port);
        }
        None => {
            println!("Please specify a command: cli or server");
        }
    }

    Ok(())
}