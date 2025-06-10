pub mod base64;
pub mod bip32;
pub mod hex;
pub mod serde;
pub mod typed_possible_values;
pub mod u256;
pub mod xprv;

use anyhow::Result;
use clap::{Arg, Command, PossibleValue};
use std::ffi::{OsStr, OsString};

pub use self::{
    base64::Base64Parser,
    bip32::Bip32PathParser,
    hex::{HexParser, HexParser16, HexParser20, HexParser32, HexParser65},
    serde::SerdeJsonFileOrLiteralParser,
    typed_possible_values::TypedPossibleValuesParser,
    u256::U256Parser,
    xprv::XprvParser,
};

pub trait FromStringParser: Clone + Send + Sync {
    type Value;
    type Error: std::fmt::Display;
    fn parse_str(&self, value: &str) -> Result<Self::Value, Self::Error>;
    fn parse_ref(
        &self,
        _cmd: &Command,
        _arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        self.parse_str(
            value
                .to_str()
                .ok_or_else(|| clap::Error::raw(clap::ErrorKind::InvalidUtf8, "\n"))?,
        )
        .map_err(|x| clap::Error::raw(clap::ErrorKind::InvalidValue, x))
    }
    fn parse(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: OsString,
    ) -> Result<Self::Value, clap::Error> {
        self.parse_ref(cmd, arg, &value)
    }
    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue<'static>> + '_>> {
        None
    }
}

macro_rules! serde_clap_parser {
    ($vis:vis $x:ident, $t:ty, $d:ty$(,)*) => {
        #[derive(Debug, Default, Copy, Clone, ::kkcli_derive::TypedValueParser)]
        $vis struct $x;

        impl $x {
            pub const fn new() -> Self {
                Self
            }
        }

        impl crate::cli::parsers::FromStringParser for $x {
            type Value = $t;
            type Error = ::serde::de::value::Error;
            fn parse_str(&self, value: &str) -> Result<Self::Value, Self::Error> {
                let deserializer = ::serde::de::IntoDeserializer::into_deserializer(value);
                <$d as ::serde_with::DeserializeAs<'_, $t>>::deserialize_as(deserializer)
            }
        }
    };
}
pub(crate) use serde_clap_parser;
