use super::serde_clap_parser;
use serde::{de::Error, Deserialize, Deserializer, Serializer};
use serde_with::{DeserializeAs, SerializeAs};
use std::borrow::Cow;

pub struct HexDef;

impl<T: AsRef<[u8]>> SerializeAs<T> for HexDef {
    fn serialize_as<S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(source))
    }
}

impl<'de, T: TryFrom<Vec<u8>>> DeserializeAs<'de, T> for HexDef {
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Cow<'de, str> as Deserialize<'de>>::deserialize(deserializer)
            .and_then(|s| hex::decode(s.strip_prefix("0x").unwrap_or(&s)).map_err(Error::custom))
            .and_then(|vec: Vec<u8>| {
                let length = vec.len();
                vec.try_into().map_err(|_e: T::Error| {
                    Error::custom(format!(
                        "Can't convert a Byte Vector of length {} to the output type.",
                        length
                    ))
                })
            })
    }
}

serde_clap_parser! {
    pub HexParser,
    Vec<u8>,
    HexDef,
}

serde_clap_parser! {
    pub HexParser16,
    [u8; 16],
    HexDef,
}

serde_clap_parser! {
    pub HexParser20,
    [u8; 20],
    HexDef,
}

serde_clap_parser! {
    pub HexParser32,
    [u8; 32],
    HexDef,
}

serde_clap_parser! {
    pub HexParser65,
    [u8; 65],
    HexDef,
}
