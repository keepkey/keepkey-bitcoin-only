use clap::builder::{PossibleValuesParser, TypedValueParser, ValueParserFactory};
use core::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct TypedPossibleValuesParser<T> {
    values: PossibleValuesParser,
    _0: PhantomData<T>,
}

impl<T> TypedPossibleValuesParser<T> {
    pub fn new(values: impl Into<PossibleValuesParser>) -> Self {
        Self {
            values: values.into(),
            _0: PhantomData,
        }
    }
}

impl<T> TypedValueParser for TypedPossibleValuesParser<T>
where
    T: Clone + Send + Sync + 'static + ValueParserFactory,
    T::Parser: TypedValueParser,
{
    type Value = <<T as ValueParserFactory>::Parser as TypedValueParser>::Value;
    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let value: std::ffi::OsString = self.values.parse_ref(cmd, arg, value)?.into();
        <T as ValueParserFactory>::value_parser().parse(cmd, arg, value)
    }
    fn parse(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: std::ffi::OsString,
    ) -> Result<Self::Value, clap::Error> {
        let value: std::ffi::OsString = self.values.parse(cmd, arg, value)?.into();
        <T as ValueParserFactory>::value_parser().parse(cmd, arg, value)
    }
    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = clap::PossibleValue<'static>> + '_>> {
        self.values.possible_values()
    }
}
