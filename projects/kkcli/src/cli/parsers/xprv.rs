use super::serde_clap_parser;
use anyhow::Result;
use bitcoin::{base58, bip32};

use serde_with::serde_conv;

serde_conv! {
    pub XprvDef,
    bip32::ExtendedPrivKey,
    |x: &bip32::ExtendedPrivKey| base58::check_encode_slice(&x.encode()),
    |x: &str| -> Result<bip32::ExtendedPrivKey, bip32::Error> {
        bip32::ExtendedPrivKey::decode(&base58::from_check(x)?)
    }
}

serde_clap_parser! {
    pub XprvParser,
    bip32::ExtendedPrivKey,
    XprvDef,
}
