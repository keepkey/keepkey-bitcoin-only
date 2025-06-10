use crate::{
    cli::{expect_field, expect_message, parsers::HexParser, types::ByteVec, CliCommand},
    messages::{self, Message},
    transport::ProtocolAdapter,
};
use anyhow::Result;
use clap::{builder::ArgGroup, ArgAction::SetTrue, Args};

/// On devices with manufacturing firmware, writes a payload to flash memory and returns the resulting Keccak-256 hash.
#[derive(Debug, Clone, Args)]
#[clap(group(ArgGroup::new("action").required(true)))]
pub struct FlashWrite {
    address: u32,
    #[clap(group = "action", value_parser = HexParser)]
    data: Option<ByteVec>,
    #[clap(long, group = "action", action = SetTrue)]
    erase: Option<bool>,
}

impl CliCommand for FlashWrite {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        let resp = expect_message!(
            Message::FlashHashResponse,
            protocol_adapter.handle(
                messages::FlashWrite {
                    address: Some(self.address),
                    data: self.data,
                    erase: self.erase,
                }
                .into()
            )
        )?;

        println!("{}", hex::encode(expect_field!(resp.data)?));

        Ok(())
    }
}
