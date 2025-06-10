use super::serde_clap_parser;

serde_clap_parser! {
    pub Base64Parser,
    Vec<u8>,
    serde_with::base64::Base64,
}
