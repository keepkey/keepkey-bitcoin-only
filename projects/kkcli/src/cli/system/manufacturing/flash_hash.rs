use crate::{
    cli::{expect_field, expect_message, parsers::HexParser, types::ByteVec, CliCommand},
    messages::{self, Message},
    transport::ProtocolAdapter,
};
use anyhow::Result;
use clap::Args;

/// On devices with manufacturing firmware, gets the Keccak-256 hash of a section of flash memory.
#[derive(Debug, Clone, Args)]
pub struct FlashHash {
    address: u32,
    length: u32,
    #[clap(short, long, value_parser = HexParser)]
    challenge: Option<ByteVec>,
}

impl CliCommand for FlashHash {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        let resp = expect_message!(
            Message::FlashHashResponse,
            protocol_adapter.handle(
                messages::FlashHash {
                    address: Some(self.address),
                    length: Some(self.length),
                    challenge: self.challenge,
                }
                .into()
            )
        )?;

        println!("{}", hex::encode(expect_field!(resp.data)?));

        Ok(())
    }
}
