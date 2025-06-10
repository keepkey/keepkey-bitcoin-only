use crate::{cli::CliCommand, messages, transport::ProtocolAdapter};
use anyhow::Result;
use clap::Args;

/// On devices with manufacturing firmware, triggers a soft reset.
#[derive(Debug, Clone, Args)]
pub struct SoftReset;

impl CliCommand for SoftReset {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        protocol_adapter.send(messages::SoftReset {}.into())?;

        Ok(())
    }
}
