use crate::{
    cli::{expect_field, expect_message, CliCommand},
    messages::{self, CoinType, Message},
    transport::ProtocolAdapter,
};
use anyhow::Result;
use clap::Args;
use core::cmp::min;

/// List all supported coin types by the device
#[derive(Debug, Clone, Args)]
pub struct ListCoins;

impl CliCommand for ListCoins {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        let resp = expect_message!(
            Message::CoinTable,
            protocol_adapter.handle(
                messages::GetCoinTable {
                    start: None,
                    end: None,
                }
                .into(),
            )
        )?;
        let (num_coins, chunk_size) = (
            *expect_field!(resp.num_coins)?,
            *expect_field!(resp.chunk_size)?,
        );
        let coin_table = (0..num_coins)
            .step_by(chunk_size.try_into()?)
            .flat_map(|start| {
                let resp = expect_message!(
                    Message::CoinTable,
                    protocol_adapter.handle(
                        messages::GetCoinTable {
                            start: Some(start),
                            end: Some(min(num_coins, start + chunk_size)),
                        }
                        .into(),
                    )
                )
                .unwrap();
                resp.table
            });

        println!(
            "{}",
            serde_json::to_string_pretty(&coin_table.collect::<Vec<CoinType>>())?
        );

        Ok(())
    }
}
