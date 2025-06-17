use std::time::Duration;

use comfy_table::{presets::UTF8_FULL, Table};
use rusb::UsbContext;
use rusb::{Context, Device, DeviceDescriptor};
use tokio::time::sleep;

const KEEPKEY_VID: u16 = 0x2b24; // KeepKey USB vendor ID

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let ctx = Context::new()?;

    loop {
        // Collect current devices
        let keepkeys: Vec<_> = ctx
            .devices()?
            .iter()
            .filter_map(|d| match d.device_descriptor() {
                Ok(desc) if desc.vendor_id() == KEEPKEY_VID => Some((d, desc)),
                _ => None,
            })
            .collect();

        render_table(&keepkeys)?;

        sleep(Duration::from_secs(2)).await;
    }
}

fn render_table(devs: &[(Device<Context>, DeviceDescriptor)]) -> anyhow::Result<()> {
    // Clear screen ANSI
    print!("\x1B[2J\x1B[H");

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(["Bus:Address", "Serial", "Product", "State"]);

    for (dev, desc) in devs {
        let addr = format!("{}:{}", dev.bus_number(), dev.address());
        let handle = dev.open();
        let serial = handle
            .as_ref()
            .ok()
            .and_then(|h| h.read_serial_number_string_ascii(&desc).ok())
            .unwrap_or_else(|| "<unknown>".to_string());
        let product = handle
            .as_ref()
            .ok()
            .and_then(|h| h.read_product_string_ascii(&desc).ok())
            .unwrap_or_else(|| "KeepKey".to_string());
        // For now we don\'t talk to the device, so state is always "Detected"
        table.add_row([addr, serial, product, "Detected".into()]);
    }

    println!("{}", table);
    Ok(())
}
