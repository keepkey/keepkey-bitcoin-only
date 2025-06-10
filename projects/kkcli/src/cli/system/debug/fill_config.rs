use crate::{cli::CliDebugCommand, messages, transport::ProtocolAdapter};
use anyhow::{anyhow, Result};
use clap::Args;

/// On a DEBUG_LINK bootloader, fills the storage sectors with a dummy data pattern.
#[derive(Debug, Clone, Args)]
pub struct DebugLinkFillConfig;

impl CliDebugCommand for DebugLinkFillConfig {
    fn handle_debug(
        self,
        _: &mut dyn ProtocolAdapter,
        debug_protocol_adapter: Option<&mut dyn ProtocolAdapter>,
    ) -> Result<()> {
        let debug_protocol_adapter = debug_protocol_adapter
            .ok_or_else(|| anyhow!("this command requires a DEBUG_LINK connection"))?;

        debug_protocol_adapter.send(messages::DebugLinkFillConfig {}.into())?;

        Ok(())
    }
}
