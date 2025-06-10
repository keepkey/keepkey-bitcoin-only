pub mod decode;
pub mod list;
mod macros;
pub mod parsers;
pub mod system;
pub mod types;
pub mod utxo;
pub mod server;

use decode::*;
use list::*;
pub(crate) use macros::*;
use system::*;
use utxo::*;
use server::*;

use crate::transport::ProtocolAdapter;
use anyhow::Result;
use clap::{ArgAction::SetTrue, Parser};

pub trait CliCommand {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()>;
}

pub trait CliDebugCommand {
    fn handle_debug(
        self,
        protocol_adapter: &mut dyn ProtocolAdapter,
        debug_protocol_adapter: Option<&mut dyn ProtocolAdapter>,
    ) -> Result<()>;
}

impl<T: CliCommand> CliDebugCommand for T {
    fn handle_debug(
        self,
        protocol_adapter: &mut dyn ProtocolAdapter,
        _: Option<&mut dyn ProtocolAdapter>,
    ) -> Result<()> {
        self.handle(protocol_adapter)
    }
}

/// Command line tool for working with KeepKey devices
#[derive(Parser, Debug, Clone)]
#[clap(version, about)]
pub struct Cli {
    /// show communication with device
    #[clap(short, long, default_value_t = false, action = SetTrue)]
    pub verbose: bool,
    /// use HID transport instead of USB (no sudo required)
    #[clap(long, default_value_t = false, action = SetTrue)]
    pub hid: bool,
    /// transport used for talking with the device
    /*#[clap(short, long, value_enum, default_value_t = TransportType::Usb)]
    pub transport: TransportType,
    /// path used by the transport (usually serial port)
    #[clap(short, long, requires = "transport")]
    pub path: Option<String>,
    /// DEBUG_LINK transport
    #[clap(long, value_enum, default_value_t = TransportType::Usb)]
    pub debuglink_transport: TransportType,
    /// path used by the DEBUG_LINK transport (usually serial port)
    #[clap(long, requires = "debuglink-transport")]
    pub debuglink_path: Option<String>,
    /// print result as json object
    #[clap(short, long, default_value_t = false)]
    pub json: bool,
    /// enable low-level debugging
    #[clap(short, long, default_value_t = false)]
    pub debug: bool,
    /// automatically press the button during user interaction prompts (on DEBUG_LINK devices only)
    #[clap(short, long, default_value_t = false)]
    pub auto_button: bool,*/
    #[clap(subcommand)]
    pub command: Subcommand,
}

// Define an empty struct for the Onboard subcommand if the macro requires it.
// This might not be strictly necessary depending on how `use_cli_subcommands!` works,
// but it's good practice to have a corresponding type.
#[derive(Parser, Debug, Clone)]
pub struct Onboard;

// Dummy implementation for CliCommand
impl CliCommand for Onboard {
    fn handle(self, _protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        // This will not be called as Onboard is handled directly in main.rs
        Ok(())
    }
}

use_cli_subcommands! {
    Onboard,
    List,
    Decode,
    Server,
    Ping,
    GetFeatures,
    ListCoins,
    ApplySettings,
    ChangePin,
    ApplyPolicy,
    GetEntropy,
    ClearSession,
    WipeDevice,
    RecoveryDevice,
    LoadDevice,
    ResetDevice,
    FirmwareUpdate,
    CipherKeyValue,
    GetPublicKey,
    GetAddress,
    SignMessage,
    VerifyMessage,
    DebugLinkGetState,
    DebugLinkFlashDump,
    DebugLinkFillConfig,
    SignIdentity,
    SignTx,
    ChangeWipeCode,
    FlashHash,
    FlashWrite,
    SoftReset,
}
