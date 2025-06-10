use super::FromStringParser;
use anyhow::{Error, Result};
use kkcli_derive::TypedValueParser;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

#[derive(Default, Debug, Clone, Copy, TypedValueParser)]
pub struct SerdeJsonFileOrLiteralParser<T: Clone + Send + Sync + 'static + DeserializeOwned>(
    PhantomData<T>,
);

impl<T: Clone + Send + Sync + 'static + DeserializeOwned> SerdeJsonFileOrLiteralParser<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Clone + Send + Sync + 'static + DeserializeOwned> FromStringParser
    for SerdeJsonFileOrLiteralParser<T>
{
    type Value = T;
    type Error = Error;
    fn parse_str(&self, value: &str) -> Result<Self::Value> {
        let value = if value.starts_with('{') || value.starts_with('[') {
            value.to_owned()
        } else {
            String::from_utf8(std::fs::read(value)?)?
        };
        Ok(serde_json::from_str(&value)?)
    }
}
