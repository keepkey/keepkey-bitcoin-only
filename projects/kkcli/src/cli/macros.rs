macro_rules! expect_message {
    ($path:path, $target:expr$(,)*) => {
        match $target {
            Ok(x) => match x {
                $path(y) => Ok(y),
                y => Err(::anyhow::anyhow!("unexpected message ({:?})", y)),
            },
            Err(x) => Err(x),
        }
    };
}
pub(crate) use expect_message;

macro_rules! expect_success {
    ($target:expr$(,)*) => {
        crate::cli::expect_message!(crate::messages::Message::Success, $target).map(|x| {
            println!("Success: {}", x.message());
            x
        })
    };
}
pub(crate) use expect_success;

macro_rules! expect_field {
    ($target:ident.$field:ident) => {{
        #[derive(Clone, Copy, Default)]
        struct TypeRef<T>(::core::marker::PhantomData<*const T>);
        impl<T> TypeRef<T> {
            const fn from_ref(_: &T) -> Self {
                Self(::core::marker::PhantomData)
            }
            fn type_name(&self) -> &'static str {
                ::core::any::type_name::<T>()
            }
            fn type_ident(&self) -> &'static str {
                let type_name = self.type_name();
                type_name
                    .rfind("::")
                    .map(|x| &type_name[(x + "::".len())..])
                    .unwrap_or(type_name)
            }
        }

        let type_ref = TypeRef::from_ref(&$target);
        $target.$field.as_ref().ok_or_else(|| {
            ::anyhow::anyhow!(
                "expected {} field in {} message",
                stringify!($field),
                type_ref.type_ident(),
            )
        })
    }};
}
pub(crate) use expect_field;

macro_rules! use_cli_subcommands {
    ($($x:ident),*$(,)*) => {
        #[derive(::clap::Subcommand, Debug, Clone)]
        #[clap(disable_help_subcommand = true, dont_collapse_args_in_usage = true)]
        pub enum Subcommand {
            $($x($x)),*
        }

        impl crate::cli::CliDebugCommand for Cli {
            fn handle_debug(self, protocol_adapter: &mut dyn crate::transport::ProtocolAdapter, debug_protocol_adapter: Option<&mut dyn crate::transport::ProtocolAdapter>) -> ::anyhow::Result<()> {
                protocol_adapter.reset()?;
                crate::cli::expect_message!(crate::messages::Message::Features, protocol_adapter.handle(crate::messages::Initialize::default().into()))?;

                match self.command {
                    $(Subcommand::$x(cmd) => crate::cli::CliDebugCommand::handle_debug(cmd, protocol_adapter, debug_protocol_adapter)),*
                }
            }
        }
    };
}
pub(crate) use use_cli_subcommands;
