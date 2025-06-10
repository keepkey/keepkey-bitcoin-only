use super::parsers::HexParser;
use crate::{cli::CliCommand, messages::Message, transport::ProtocolAdapter};
use anyhow::{anyhow, Result};
use clap::Args;

/// Decode a raw message
#[derive(Debug, Clone, Args)]
pub struct Decode {
    #[clap(required = true, multiple = false, value_parser = HexParser)]
    data: Vec<Vec<u8>>,
}

impl Decode {
    pub fn handle(self) -> Result<()> {
        let mut data = self.data[0].clone();

        if !data.is_empty() && data[0] == b'?' {
            data.remove(0);
        }
        let msg = Message::decode(&mut data.as_slice()).map_err(|x| anyhow!(x))?;

        println!("{:?}", msg);
        Ok(())
    }
}

impl CliCommand for Decode {
    fn handle(self, _: &mut dyn ProtocolAdapter) -> Result<()> {
        unreachable!();
    }
}
