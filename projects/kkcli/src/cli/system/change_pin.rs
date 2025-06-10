use crate::{
    cli::{expect_success, CliCommand},
    messages,
    transport::ProtocolAdapter,
};
use anyhow::Result;
use clap::{ArgAction::SetTrue, Args};

/// Set or remove PIN protection
#[derive(Debug, Clone, Args)]
pub struct ChangePin {
    #[clap(short, long, action = SetTrue)]
    remove: Option<bool>,
}

impl CliCommand for ChangePin {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        expect_success!(protocol_adapter.with_standard_handler().handle(
            messages::ChangePin {
                remove: self.remove
            }
            .into()
        ))?;

        Ok(())
    }
}
