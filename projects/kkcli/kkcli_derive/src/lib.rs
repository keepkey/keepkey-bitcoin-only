use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{parse_macro_input, parse_str, Attribute, DeriveInput, Lit};

#[proc_macro_derive(TypedValueParser)]
pub fn derive_typed_value_parser(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, generics, ..
    } = parse_macro_input!(input);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::clap::builder::TypedValueParser for #ident #ty_generics #where_clause {
            type Value = <Self as crate::cli::parsers::FromStringParser>::Value;
            fn parse_ref(
                &self,
                cmd: &clap::Command,
                arg: Option<&clap::Arg>,
                value: &std::ffi::OsStr,
            ) -> Result<Self::Value, clap::Error> {
                <Self as crate::cli::parsers::FromStringParser>::parse_ref(self, cmd, arg, value)
            }
            fn parse(
                &self,
                cmd: &clap::Command,
                arg: Option<&clap::Arg>,
                value: std::ffi::OsString,
            ) -> Result<Self::Value, clap::Error> {
                <Self as crate::cli::parsers::FromStringParser>::parse(self, cmd, arg, value)
            }
            fn possible_values(
                &self,
            ) -> Option<Box<dyn Iterator<Item = clap::PossibleValue<'static>> + '_>> {
                <Self as crate::cli::parsers::FromStringParser>::possible_values(self)
            }
        }
    };
    output.into()
}

#[proc_macro_derive(SerdeAsRemote)]
pub fn derive_serde_as_remote(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        attrs,
        generics,
        ..
    } = parse_macro_input!(input);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let ty = attrs
        .into_iter()
        .find(|x: &Attribute| x.path.is_ident("serde"))
        .expect("expected #[serde]");
    let ty = ty
        .tokens
        .into_iter()
        .find_map(|x| match x {
            TokenTree::Group(x) => x
                .stream()
                .into_iter()
                .collect_vec()
                .split(|x| matches!(x, TokenTree::Punct(x) if x.as_char() == ','))
                .find_map(|x| {
                    if x.len() == 3
                        && matches!(&x[0], TokenTree::Ident(x) if x.to_string() == "remote")
                        && matches!(&x[1], TokenTree::Punct(x) if x.as_char() == '=')
                    {
                        if let TokenTree::Literal(x) = &x[2] {
                            Some(x.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }),
            _ => None,
        })
        .expect("expected #[serde(remote = <type>)]");
    let ty = match parse_str::<syn::Lit>(&ty).expect("expected #[serde(remote = \"<type>\")]") {
        Lit::Str(x) => x.parse::<syn::Path>().ok(),
        _ => None,
    }
    .expect("expected #[serde(remote = \"<type>\")], where type is a valid path");

    let output = quote! {
        impl #impl_generics ::serde_with::SerializeAs<#ty #ty_generics> for #ident #ty_generics #where_clause {
            fn serialize_as<S: serde::Serializer>(
                value: &#ty #ty_generics,
                serializer: S,
            ) -> Result<S::Ok, S::Error> {
                #ident::serialize(value, serializer)
            }
        }

        impl<'de> #impl_generics ::serde_with::DeserializeAs<'de, #ty #ty_generics> for #ident #ty_generics #where_clause {
            fn deserialize_as<D: serde::Deserializer<'de>>(deserializer: D) -> Result<#ty #ty_generics, D::Error> {
                #ident::deserialize(deserializer)
            }
        }
    };

    output.into()
}

#[proc_macro_derive(SerdeAsSelf)]
pub fn derive_serde_as_self(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, generics, ..
    } = parse_macro_input!(input);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let output = quote! {
        impl #impl_generics ::serde_with::SerializeAs<Self> for #ident #ty_generics #where_clause {
            fn serialize_as<S: serde::Serializer>(
                value: &Self,
                serializer: S,
            ) -> Result<S::Ok, S::Error> {
                <Self as ::serde::Serialize>::serialize(value, serializer)
            }
        }

        impl<'de> #impl_generics ::serde_with::DeserializeAs<'de, Self> for #ident #ty_generics #where_clause {
            fn deserialize_as<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                <Self as ::serde::Deserialize>::deserialize(deserializer)
            }
        }
    };

    output.into()
}
