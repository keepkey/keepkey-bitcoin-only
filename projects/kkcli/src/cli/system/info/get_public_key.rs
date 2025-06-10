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
use clap::{ArgAction::SetTrue, Args};

/// Get public key of given path
#[derive(Debug, Clone, Args)]
pub struct GetPublicKey {
    /// BIP-32 path to key
    #[clap(value_parser = Bip32PathParser)]
    address: Bip32Path,
    #[clap(long)]
    ecdsa_curve_name: Option<String>,
    /// Confirm address on device screen
    #[clap(short = 'd', long, action = SetTrue)]
    show_display: Option<bool>,
    #[clap(short, long)]
    coin_name: Option<String>,
    #[clap(value_enum, short = 't', long)]
    script_type: Option<ScriptType>,
}

impl CliCommand for GetPublicKey {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        let resp = expect_message!(
            Message::PublicKey,
            protocol_adapter.with_standard_handler().handle(
                messages::GetPublicKey {
                    address_n: self.address.into(),
                    ecdsa_curve_name: self.ecdsa_curve_name,
                    show_display: self.show_display,
                    coin_name: self.coin_name,
                    script_type: self.script_type.map(|x| x.into()),
                }
                .into(),
            )
        )?;

        println!("{}", expect_field!(resp.xpub)?);

        Ok(())
    }
}
