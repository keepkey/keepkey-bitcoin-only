use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod commands;

use commands::{list::ListCommand, test::TestCommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "kkcli-v2")]
#[command(about = "KeepKey CLI v2 - Enhanced device management with keepkey-rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List connected KeepKey devices with detailed information
    List(ListCommand),
    /// Test device communication and bootloader detection
    Test(TestCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(if cli.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();
    
    println!("{}", "KeepKey CLI v2 - Enhanced Device Management".bright_blue().bold());
    println!("{}", "Powered by keepkey-rust".dimmed());
    println!();
    
    match cli.command {
        Commands::List(cmd) => cmd.execute().await,
        Commands::Test(cmd) => cmd.execute().await,
    }
} 