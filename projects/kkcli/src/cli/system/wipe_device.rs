use crate::{
    cli::{expect_success, CliCommand},
    messages,
    transport::ProtocolAdapter,
};
use anyhow::Result;
use clap::Args;

/// Reset device to factory defaults and remove all private data.
#[derive(Debug, Clone, Args)]
pub struct WipeDevice;

impl CliCommand for WipeDevice {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        expect_success!(protocol_adapter
            .with_standard_handler()
            .handle(messages::WipeDevice::default().into()))?;

        Ok(())
    }
}
