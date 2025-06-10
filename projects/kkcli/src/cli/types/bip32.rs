use anyhow::{bail, Error};
use kkcli_derive::SerdeAsSelf;
use lazy_static::lazy_static;
use regex::Regex;
use schemars::JsonSchema;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Default, SerializeDisplay, DeserializeFromStr, SerdeAsSelf, JsonSchema)]
#[schemars(transparent)]
pub struct Bip32Path(#[schemars(with = "String", regex(pattern = r"^m(/[0-9]+'?)*$"))] Vec<u32>);

impl AsRef<[u32]> for Bip32Path {
    fn as_ref(&self) -> &[u32] {
        &self.0
    }
}

impl From<&[u32]> for Bip32Path {
    fn from(x: &[u32]) -> Self {
        Self(x.to_vec())
    }
}

impl From<Vec<u32>> for Bip32Path {
    fn from(x: Vec<u32>) -> Self {
        Self(x)
    }
}

impl From<Bip32Path> for Vec<u32> {
    fn from(x: Bip32Path) -> Self {
        x.0
    }
}

impl FromStr for Bip32Path {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref BIP32_PATH_REGEX: Regex = Regex::new(r"^m(/\d+'?)*$").unwrap();
            static ref BIP32_PATH_SEGMENT_REGEX: Regex =
                Regex::new(r"/(?P<index>\d+)(?P<hardened>'?)").unwrap();
        }

        if !BIP32_PATH_REGEX.is_match(value) {
            bail!("value must be a valid BIP-32 path (for example, m/44'/60'/0'/0/0)\n")
        }
        let mut out = Vec::<u32>::new();
        for item in BIP32_PATH_SEGMENT_REGEX.captures_iter(value) {
            let mut index: u32 = item["index"].parse::<u32>()?;
            if index >= 0x80000000 {
                bail!("index must be less than 0x80000000")
            }
            if &item["hardened"] == "'" {
                index = index.checked_add(0x80000000).unwrap();
            }
            out.push(index);
        }
        Ok(out.into())
    }
}

impl Display for Bip32Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in self.0.iter().copied() {
            if i >= 0x80000000 {
                write!(f, "/{}'", i - 0x80000000)?;
            } else {
                write!(f, "/{}", i)?;
            }
        }

        Ok(())
    }
}
