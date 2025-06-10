Below is a worked recipe for an interactive KeepKey on-boarding wizard built with the inquire crate. Each step shows the idea first, then a tiny code fragment you can drop into main.rs (or split into modules later). All hardware I/O is stubbed with helper functions so you can focus on the CLI flow first.

0 Project skeleton
console
Copy
Edit
cargo new keepkey-wizard
cd keepkey-wizard
Cargo.toml

toml
Copy
Edit
[dependencies]
inquire  = "0.7"      # prompts, select boxes, confirm    :contentReference[oaicite:0]{index=0}
hidapi   = "2.6"      # USB HID detection (KeepKey = VID 0x2b24)   :contentReference[oaicite:1]{index=1}
semver   = "1"        # version parsing / comparison
anyhow   = "1"
1 Model the wizard state
rust
Copy
Edit
use semver::Version;

#[derive(Clone, Debug)]
pub enum DeviceState {
NoDevice,
Normal {
fw: Version,
bl: Version,
},
Updater,         // version numbers unknown
}

#[derive(Default)]
struct WizardCtx {
remembered_fw: Option<Version>,
remembered_bl: Option<Version>,
}
Why? – Once the user re-plugs in up-dater mode we can no longer read the versions, so we cache them in WizardCtx when the device is still in Normal mode.

2 Detect the current device state
rust
Copy
Edit
fn detect_device() -> anyhow::Result<DeviceState> {
let api = hidapi::HidApi::new()?;          // <-- stub real VID/PID checks
for dev in api.device_list() {
match (dev.vendor_id(), dev.product_id()) {
(0x2b24, 0x0002) => {
// Normal mode – pretend we queried firmware via protobuf
return Ok(DeviceState::Normal {
fw: "7.3.2".parse()?,
bl: "2.0.1".parse()?,
});
}
(0x2b24, 0x0003) => return Ok(DeviceState::Updater),
_ => {}
}
}
Ok(DeviceState::NoDevice)
}
3 Step-by-step flow with inquire
3.1 No device connected
rust
Copy
Edit
use inquire::{Select, Confirm};

fn step_no_device() -> anyhow::Result<()> {
println!("🔌  No KeepKey detected.");
let choice = Select::new(
"What do you want to do?",
vec!["Retry", "Run troubleshooter", "Quit"],
)
.prompt()?;

    match choice.as_str() {
        "Retry" => {}
        "Run troubleshooter" => run_troubleshooter()?,
        "Quit" => std::process::exit(0),
        _ => unreachable!(),
    }
    Ok(())
}
3.2 Normal mode (versions known)
rust
Copy
Edit
const LATEST_FW: &str = "8.0.0";
const LATEST_BL: &str = "2.2.0";

fn step_normal(ctx: &mut WizardCtx, fw: Version, bl: Version) -> anyhow::Result<()> {
println!("🟢  Device firmware {fw} | bootloader {bl}");

    // remember for later
    ctx.remembered_fw = Some(fw.clone());
    ctx.remembered_bl = Some(bl.clone());

    if fw < Version::parse(LATEST_FW)? || bl < Version::parse(LATEST_BL)? {
        if Confirm::new("Update available – enter updater mode now?")
            .with_default(true)
            .prompt()?
        {
            println!("➡️  Please hold the button while plugging in to enter updater…");
            wait_for_replug()?;                // simple timeout loop around detect_device()
        }
    } else {
        println!("✅  Device is up-to-date.");
    }
    Ok(())
}
3.3 Updater mode (versions unknown)
rust
Copy
Edit
use inquire::{MultiSelect, Text};

fn step_updater(ctx: &WizardCtx) -> anyhow::Result<()> {
println!("🛠  Device in updater mode.");

    // Pick operations
    let ops = MultiSelect::new(
        "Select the tasks you wish to perform:",
        vec!["Bootloader → latest", "Firmware → latest", "Firmware → custom file"],
    )
    .prompt()?;

    if ops.contains(&"Bootloader → latest".to_string()) {
        do_bootloader_update(LATEST_BL)?;
    }

    if ops.contains(&"Firmware → latest".to_string()) {
        do_firmware_update(LATEST_FW)?;
    }

    if ops.contains(&"Firmware → custom file".to_string()) {
        let path = Text::new("Path to signed firmware file:").prompt()?;
        do_firmware_update_from_file(&path)?;
    }

    // Use remembered versions for nice summary
    if let (Some(old_fw), Some(old_bl)) = (&ctx.remembered_fw, &ctx.remembered_bl) {
        println!(
            "🎉  Success!  {old_bl} → {LATEST_BL}, {old_fw} → {LATEST_FW}"
        );
    } else {
        println!("🎉  Success!  Device updated.");
    }
    Ok(())
}
4 Putting it together
rust
Copy
Edit
fn main() -> anyhow::Result<()> {
let mut ctx = WizardCtx::default();

    loop {
        match detect_device()? {
            DeviceState::NoDevice => step_no_device()?,
            DeviceState::Normal { fw, bl } => step_normal(&mut ctx, fw, bl)?,
            DeviceState::Updater => step_updater(&ctx)?,
        }
        // Small debounce so we don’t hammer USB every iteration
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
}
5 Next touches
Add-on	Why
indicatif progress bars while flashing firmware/bootloader
structured logging with tracing so users can --verbose
state machine crate (typestate, rust-fsm) if you want compiler-checked transitions

With these bite-sized blocks you get:

Clear states (NoDevice, Normal, Updater)

Memory of old versions via WizardCtx

Friendly prompts with inquire that guide the user the whole way

Compile, stub the USB helpers, then start filling in the real KeepKey protocol calls when you’re ready. Happy shipping!