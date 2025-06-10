use crate::{
    cli::{expect_success, CliCommand},
    messages,
    transport::ProtocolAdapter,
};
use anyhow::Result;
use clap::{ArgAction::SetTrue, Args};

/// Set or remove wipe code protection
#[derive(Debug, Clone, Args)]
pub struct ChangeWipeCode {
    #[clap(short, long, action = SetTrue)]
    remove: Option<bool>,
}

impl CliCommand for ChangeWipeCode {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        expect_success!(protocol_adapter.with_standard_handler().handle(
            messages::ChangeWipeCode {
                remove: self.remove
            }
            .into()
        ))?;

        Ok(())
    }
}
