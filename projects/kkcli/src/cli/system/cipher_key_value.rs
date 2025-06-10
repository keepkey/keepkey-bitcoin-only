use crate::{
    cli::{
        expect_field, expect_message,
        parsers::{Base64Parser, Bip32PathParser, HexParser16},
        types::{Bip32Path, ByteVec},
        CliCommand,
    },
    messages::{self, Message},
    transport::ProtocolAdapter,
};
use anyhow::Result;
use clap::{ArgAction::SetTrue, Args};

/// Encrypt/decrypt key/value pairs
#[derive(Debug, Clone, Args)]
pub struct CipherKeyValue {
    /// BIP-32 path to the encrypting/decrypting key
    #[clap(value_parser = Bip32PathParser)]
    address: Bip32Path,
    /// a key name, which will be mixed into the encryption key
    key: String,
    /// plaintext or ciphertext, in base64; length must be a multiple of 16 bytes
    #[clap(value_parser = Base64Parser)]
    value: ByteVec,
    /// decrypt ciphertext instead of encrypting plaintext
    #[clap(short, long, action = SetTrue)]
    decrypt: Option<bool>,
    /// don't encrypt without prompting the user (setting must match for both encryption and decryption)
    #[clap(long, action = SetTrue)]
    ask_on_encrypt: Option<bool>,
    /// don't decrypt without prompting the user (setting must match for both encryption and decryption)
    #[clap(long, action = SetTrue)]
    ask_on_decrypt: Option<bool>,
    /// 16-byte initialization vector for AES-256-CBC
    #[clap(long, value_parser = HexParser16)]
    iv: Option<[u8; 16]>,
}

impl CliCommand for CipherKeyValue {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        let resp = expect_message!(
            Message::CipheredKeyValue,
            protocol_adapter.with_standard_handler().handle(
                messages::CipherKeyValue {
                    address_n: self.address.into(),
                    key: Some(self.key),
                    value: Some(self.value),
                    encrypt: Some(self.decrypt != Some(true)),
                    ask_on_encrypt: self.ask_on_encrypt,
                    ask_on_decrypt: self.ask_on_decrypt,
                    iv: self.iv.map(|x| x.to_vec()),
                }
                .into(),
            )
        )?;

        println!("{}", base64::encode(expect_field!(resp.value)?));

        Ok(())
    }
}
