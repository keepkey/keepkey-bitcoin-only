use super::serde_clap_parser;

use serde_with::serde_conv;

serde_conv! {
    pub U256Def,
    U256,
    |x: &U256| x.to_string(),
    |x: &str| -> Result<_, _> {
        U256::from_dec_str(x)
    }
}

serde_clap_parser! {
    pub U256Parser,
    U256,
    U256Def,
}
