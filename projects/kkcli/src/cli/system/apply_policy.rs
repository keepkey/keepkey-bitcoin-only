use crate::{
    cli::{expect_success, CliCommand},
    messages,
    transport::ProtocolAdapter,
};
use anyhow::Result;
use clap::Args;

/// Apply a policy
#[derive(Debug, Clone, Args)]
pub struct ApplyPolicy {
    #[clap(short = 'o', long, default_value = "")]
    policy_name: String,
    #[clap(short = 'c', long, default_value_t = true)]
    enabled: bool,
}

impl CliCommand for ApplyPolicy {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        expect_success!(protocol_adapter.with_standard_handler().handle(
            messages::ApplyPolicies {
                policy: vec![messages::PolicyType {
                    policy_name: Some(self.policy_name),
                    enabled: Some(self.enabled),
                }],
            }
            .into(),
        ))?;

        Ok(())
    }
}
