use crate::transport::ProtocolAdapter;
use anyhow::Result;
use clap::Parser;

/// Start the KeepKey CLI server with REST API and MCP support
#[derive(Parser, Debug, Clone)]
pub struct Server {
    /// Port to run the server on
    #[clap(short, long, default_value = "1646")]
    pub port: u16,
    
    /// Enable verbose logging
    #[clap(short, long)]
    pub verbose: bool,
    
    /// Keep the server running (default behavior)
    #[clap(long)]
    pub daemon: bool,
    
    // Removed allow_mock field as per Pioneer Guild Guidelines - "NEVER MOCK ANYTHING"
}

impl super::CliCommand for Server {
    fn handle(self, _protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        // This is a special case - the server command doesn't need a protocol adapter
        // It will be handled directly in main.rs using tokio runtime
        println!("Server command should be handled in main.rs with async runtime");
        Ok(())
    }
}

impl Server {
    pub async fn run(self) -> Result<()> {
        // Only initialize tracing if it hasn't been initialized already
        // This avoids the "Unable to install global subscriber" error
        if std::env::var("RUST_LOG").is_err() {
            // Set the environment variable based on verbose flag
            if self.verbose {
                std::env::set_var("RUST_LOG", "kkcli=debug,axum=debug,tower_http=debug");
            } else {
                std::env::set_var("RUST_LOG", "kkcli=info,axum=info");
            }
            // The main.rs file will handle the actual initialization
        }
        
        println!("Starting KeepKey CLI server on port {}", self.port);
        println!("Press Ctrl+C to stop the server");
        
        // Start the server - no more allow_mock as per Pioneer Guild Guidelines
        crate::server::start_server(self.port).await?;
        
        Ok(())
    }
} 