mod bip32;

pub use bip32::Bip32Path;

use crate::messages;
use clap::ValueEnum;


/// This type alias keeps clap's derive macro from misinterpreting an arg which takes many bytes as a repeated arg where each instance takes one byte.
pub type ByteVec = Vec<u8>;

pub trait IntoBigEndian {
    fn into_big_endian(self) -> Vec<u8>;
}



#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum ScriptType {
    P2pkh,
    P2wpkh,
    P2shP2wpkh,
}

impl From<ScriptType> for messages::InputScriptType {
    fn from(x: ScriptType) -> Self {
        match x {
            ScriptType::P2pkh => messages::InputScriptType::Spendaddress,
            ScriptType::P2wpkh => messages::InputScriptType::Spendwitness,
            ScriptType::P2shP2wpkh => messages::InputScriptType::Spendp2shwitness,
        }
    }
}

impl From<ScriptType> for i32 {
    fn from(x: ScriptType) -> Self {
        Into::<messages::InputScriptType>::into(x) as i32
    }
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum EosPublicKeyKind {
    Eos,
    EosK1,
    EosR1,
}

impl From<EosPublicKeyKind> for messages::EosPublicKeyKind {
    fn from(x: EosPublicKeyKind) -> Self {
        match x {
            EosPublicKeyKind::Eos => messages::EosPublicKeyKind::Eos,
            EosPublicKeyKind::EosK1 => messages::EosPublicKeyKind::EosK1,
            EosPublicKeyKind::EosR1 => messages::EosPublicKeyKind::EosR1,
        }
    }
}

impl From<EosPublicKeyKind> for i32 {
    fn from(x: EosPublicKeyKind) -> Self {
        Into::<messages::EosPublicKeyKind>::into(x) as i32
    }
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum OutputAddressType {
    Spend,
    Transfer,
    Change,
}

impl From<OutputAddressType> for messages::OutputAddressType {
    fn from(x: OutputAddressType) -> Self {
        match x {
            OutputAddressType::Spend => messages::OutputAddressType::Spend,
            OutputAddressType::Transfer => messages::OutputAddressType::Transfer,
            OutputAddressType::Change => messages::OutputAddressType::Change,
        }
    }
}

impl From<OutputAddressType> for i32 {
    fn from(x: OutputAddressType) -> Self {
        Into::<messages::OutputAddressType>::into(x) as i32
    }
}

/*#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum TransportType {
    Usb,
    Serial,
    Pipe,
    Socket,
    Bridge,
    Udp,
    WebUsb,
}*/
