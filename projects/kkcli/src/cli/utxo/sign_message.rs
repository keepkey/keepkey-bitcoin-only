use crate::{
    cli::{
        expect_field, expect_message,
        parsers::Bip32PathParser,
        types::{Bip32Path, ScriptType},
        CliCommand,
    },
    messages::{self, Message},
    transport::ProtocolAdapter,
};
use anyhow::Result;
use clap::Args;

/// Sign a message, Bitcoin-style
#[derive(Debug, Clone, Args)]
pub struct SignMessage {
    /// BIP-32 path to signing key
    #[clap(short = 'n', long, value_parser = Bip32PathParser, default_value = "m/44'/0'/0'/0/0")]
    address: Bip32Path,
    message: String,
    #[clap(short, long)]
    coin_name: Option<String>,
    #[clap(value_enum, short = 't', long)]
    script_type: Option<ScriptType>,
}

impl CliCommand for SignMessage {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        let resp = expect_message!(
            Message::MessageSignature,
            protocol_adapter.with_standard_handler().handle(
                messages::SignMessage {
                    address_n: self.address.into(),
                    message: self.message.into_bytes(),
                    coin_name: self.coin_name,
                    script_type: self.script_type.map(|x| x.into()),
                }
                .into(),
            )
        )?;

        println!("Address: {}", expect_field!(resp.address)?);
        println!(
            "Signature: {}",
            base64::encode(expect_field!(resp.signature)?)
        );

        Ok(())
    }
}
