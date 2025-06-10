use crate::{
    cli::{expect_message, CliCommand},
    messages::{self, Message},
    transport::ProtocolAdapter,
};
use anyhow::Result;
use clap::Args;
use core::time::Duration;

/// Retrieve device features and settings
#[derive(Debug, Clone, Args)]
pub struct GetFeatures;

impl CliCommand for GetFeatures {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        let features = expect_message!(
            Message::Features,
            protocol_adapter.handle(messages::GetFeatures::default().into())
        )?;

        if let Some(label) = features.label {
            println!("label:\t{}", label);
        }
        if let Some(vendor) = features.vendor {
            println!("vendor:\t\t\t{}", vendor);
        }
        if let Some(model) = features.model {
            println!("model:\t\t\t{}", model);
        }
        if let Some(firmware_variant) = features.firmware_variant {
            println!("variant:\t\t{}", firmware_variant);
        }
        if let Some(device_id) = features.device_id {
            println!("device id:\t\t{}", device_id);
        }
        if let Some(language) = features.language {
            println!("language:\t{}", language);
        }

        println!();
        let bootloader_mode = features.bootloader_mode.unwrap_or(false);
        if bootloader_mode {
            println!("(device is in bootloader mode)");
            println!();
        }
        println!(
            "{} version:\t{}.{}.{}",
            if bootloader_mode {
                "bootloader"
            } else {
                "firmware"
            },
            features
                .major_version
                .map_or_else(|| "?".to_owned(), |x| x.to_string()),
            features
                .minor_version
                .map_or_else(|| "?".to_owned(), |x| x.to_string()),
            features
                .patch_version
                .map_or_else(|| "?".to_owned(), |x| x.to_string())
        );
        if let Some(revision) = features.revision {
            println!(
                "revision:\t\t{}",
                std::str::from_utf8(&revision)
                    .map_or_else(|_| hex::encode(&revision), |x| x.to_owned())
            );
        }
        if let Some(firmware_hash) = features.firmware_hash {
            println!("firmware hash:\t\t{}", hex::encode(firmware_hash));
        }
        if let Some(bootloader_hash) = features.bootloader_hash {
            println!("bootloader hash:\t{}", hex::encode(bootloader_hash));
        }

        println!();
        if let Some(initialized) = features.initialized {
            println!("initialized:\t\t{}", initialized);
        }
        match features.imported {
            Some(false) => {
                println!("\t\t\t(keys were generated or recovered on-device)")
            }
            Some(true) => println!("\t\t\t(keys were imported from a computer)"),
            _ => (),
        }
        if let Some(true) = features.no_backup {
            println!("\t\t\t(keys were not backed up during setup)")
        }
        if let Some(pin_protection) = features.pin_protection {
            println!("PIN protection:\t\t{}", pin_protection);
        }
        if features.pin_cached == Some(true) {
            println!("\t\t\t(device currently unlocked)");
        }
        if let Some(passphrase_protection) = features.passphrase_protection {
            println!("passphrase protection:\t{}", passphrase_protection);
        }
        if features.passphrase_cached == Some(true) {
            println!("\t\t\t(passprase is currently cached)");
        }
        if let Some(wipe_code_protection) = features.wipe_code_protection {
            println!("wipe code protection:\t{}", wipe_code_protection);
        }
        if let Some(auto_lock_delay_ms) = features.auto_lock_delay_ms {
            println!(
                "screensaver delay:\t{}",
                humantime::format_duration(Duration::from_millis(auto_lock_delay_ms.into()))
            );
        }

        println!();
        println!("policies:");
        if features.policies.is_empty() {
            println!("\t(none)");
        }
        for policy in features.policies {
            println!(
                "\t{}:\t{}",
                policy.policy_name(),
                if policy.enabled() {
                    "enabled"
                } else {
                    "disabled"
                }
            );
        }

        Ok(())
    }
}
