use crate::{
    cli::{
        expect_message,
        parsers::Bip32PathParser,
        types::{Bip32Path, ScriptType},
        CliCommand,
    },
    messages::{self, Message},
    transport::ProtocolAdapter,
};
use anyhow::Result;
use clap::{ArgAction::SetTrue, Args};

/// Get bitcoin address in base58 encoding
#[derive(Debug, Clone, Args)]
pub struct GetAddress {
    /// BIP-32 path to key
    #[clap(short = 'n', long, value_parser = Bip32PathParser, default_value = "m/44'/0'/0'/0/0")]
    address: Bip32Path,
    #[clap(short, long)]
    coin_name: Option<String>,
    #[clap(value_enum, short = 't', long)]
    script_type: Option<ScriptType>,
    /// Confirm address on device screen
    #[clap(short = 'd', long, action = SetTrue)]
    show_display: Option<bool>,
}

impl CliCommand for GetAddress {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        let resp = expect_message!(
            Message::Address,
            protocol_adapter.with_standard_handler().handle(
                messages::GetAddress {
                    coin_name: self.coin_name,
                    address_n: self.address.into(),
                    script_type: self.script_type.map(|x| x.into()),
                    show_display: self.show_display,
                    multisig: None,
                }
                .into()
            ),
        )?;

        println!("{}", resp.address);

        Ok(())
    }
}
