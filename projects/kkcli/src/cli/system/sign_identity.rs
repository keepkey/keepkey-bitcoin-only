use crate::{
    cli::{expect_field, expect_message, parsers::HexParser, types::ByteVec, CliCommand},
    messages::{self, Message},
    transport::ProtocolAdapter,
};
use anyhow::Result;
use clap::Args;
use url::Url;

/// Ask device to sign an identity challenge.
///
/// Supports SSH and GPG when using identity urls beginning with "ssh:" or "gpg:".
#[derive(Debug, Clone, Args)]
pub struct SignIdentity {
    /// identity url
    url: String,
    /// identity index
    #[clap(short, long)]
    index: Option<u32>,
    /// challenge shown on display (e.g. date+time). (Ignored when signing with SSH or GPG identities.)
    #[clap(short = 'v', long)]
    challenge_visual: Option<String>,
    /// non-visible challenge
    #[clap(value_parser = HexParser)]
    challenge_hidden: ByteVec,
    /// ECDSA curve name to use
    #[clap(long)]
    ecdsa_curve_name: Option<String>,
}

impl CliCommand for SignIdentity {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        let url = Some(self.url)
            .filter(|x| !(*x).is_empty())
            .map(|x| Url::parse(&x))
            .transpose()?;

        let resp = expect_message!(
            Message::SignedIdentity,
            protocol_adapter.with_standard_handler().handle(
                messages::SignIdentity {
                    identity: Some(messages::IdentityType {
                        proto: url.as_ref().map(|x| x.scheme().to_string()),
                        user: url
                            .as_ref()
                            .map(|x| x.username())
                            .filter(|x| !(*x).is_empty())
                            .map(|x| x.to_string()),
                        host: url
                            .as_ref()
                            .and_then(|x| x.host_str())
                            .map(|x| x.to_string()),
                        port: url.as_ref().and_then(|x| x.port()).map(|x| x.to_string()),
                        path: url
                            .as_ref()
                            .map(|x| x.path())
                            .filter(|x| !(*x).is_empty())
                            .map(|x| x.to_string()),
                        index: self.index,
                    }),
                    challenge_hidden: Some(self.challenge_hidden),
                    challenge_visual: self.challenge_visual,
                    ecdsa_curve_name: self.ecdsa_curve_name,
                }
                .into(),
            )
        )?;

        if let Some(ref address) = resp.address {
            println!("Address:\t{}", address);
        }
        println!(
            "Public Key:\t{}",
            hex::encode(expect_field!(resp.public_key)?)
        );
        println!(
            "Signature:\t{}",
            hex::encode(expect_field!(resp.signature)?)
        );

        Ok(())
    }
}
