use crate::{
    cli::{expect_message, CliCommand},
    messages::{self, Message},
    transport::ProtocolAdapter,
};
use anyhow::Result;
use clap::Args;

/// Get entropy from the device
#[derive(Debug, Clone, Args)]
pub struct GetEntropy {
    size: u32,
}

impl CliCommand for GetEntropy {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        let resp = expect_message!(
            Message::Entropy,
            protocol_adapter
                .with_standard_handler()
                .handle(messages::GetEntropy { size: self.size }.into())
        )?;
        println!("{}", hex::encode(resp.entropy));

        Ok(())
    }
}
