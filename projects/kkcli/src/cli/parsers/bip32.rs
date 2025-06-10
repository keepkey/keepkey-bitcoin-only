use super::serde_clap_parser;
use crate::cli::types::Bip32Path;

serde_clap_parser! {
    pub Bip32PathParser,
    Bip32Path,
    Bip32Path,
}
