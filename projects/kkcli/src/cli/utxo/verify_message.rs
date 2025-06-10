use crate::{
    cli::{parsers::Base64Parser, types::ByteVec, CliCommand},
    messages::{self, Message},
    transport::ProtocolAdapter,
};
use anyhow::{bail, Result};
use clap::Args;

/// Verify a message, Bitcoin-style
#[derive(Debug, Clone, Args)]
pub struct VerifyMessage {
    /// address to verify the signature against
    address: String,
    message: String,
    /// signature to verify, in base64
    #[clap(value_parser = Base64Parser)]
    signature: ByteVec,
    /// coin which matches the address's type
    #[clap(short, long)]
    coin_name: Option<String>,
}

impl CliCommand for VerifyMessage {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        match protocol_adapter.with_standard_handler().handle(
            messages::VerifyMessage {
                address: Some(self.address),
                signature: Some(self.signature),
                message: Some(self.message.into_bytes()),
                coin_name: self.coin_name,
            }
            .into(),
        )? {
            Message::Success(_) => (),
            Message::Failure(_) => (),
            x => bail!("unexpected message ({:?})", x),
        }

        Ok(())
    }
}
