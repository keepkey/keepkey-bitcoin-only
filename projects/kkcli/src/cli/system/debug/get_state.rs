use crate::{
    cli::{expect_message, CliDebugCommand},
    messages::{self, Message},
    transport::ProtocolAdapter,
};
use anyhow::{anyhow, Result};
use clap::Args;

/// On DEBUG_LINK firmware, dumps various internal sensitive paramters.
#[derive(Debug, Clone, Args)]
pub struct DebugLinkGetState;

impl CliDebugCommand for DebugLinkGetState {
    fn handle_debug(
        self,
        _: &mut dyn ProtocolAdapter,
        debug_protocol_adapter: Option<&mut dyn ProtocolAdapter>,
    ) -> Result<()> {
        let debug_protocol_adapter = debug_protocol_adapter
            .ok_or_else(|| anyhow!("this command requires a DEBUG_LINK connection"))?;

        let resp = expect_message!(
            Message::DebugLinkState,
            debug_protocol_adapter.handle(messages::DebugLinkGetState {}.into())
        )?;

        println!("{:#?}", resp);

        Ok(())
    }
}
