use crate::{
    cli::{expect_success, CliCommand},
    messages,
    transport::ProtocolAdapter,
};
use anyhow::Result;
use clap::Args;

/// Set new wallet label
#[derive(Debug, Clone, Args)]
pub struct ApplySettings {
    /// screensaver timeout
    #[clap(short, long)]
    auto_lock_delay_ms: Option<u32>,
    #[clap(long)]
    label: Option<String>,
    #[clap(long)]
    language: Option<String>,
    /// Enable BIP39 passphrase protection.
    #[clap(long)]
    use_passphrase: Option<bool>,
    #[clap(short, long)]
    u2f_counter: Option<u32>,
}

impl CliCommand for ApplySettings {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        let req = messages::ApplySettings {
            auto_lock_delay_ms: self.auto_lock_delay_ms,
            label: self.label,
            language: self.language,
            use_passphrase: self.use_passphrase,
            u2f_counter: self.u2f_counter,
        };

        expect_success!(protocol_adapter.with_standard_handler().handle(req.into()))?;

        Ok(())
    }
}
