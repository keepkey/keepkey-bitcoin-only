use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// Core trait that defines all device operations needed by the REST API.
/// This allows complete separation between the REST API and hardware implementation.
pub trait DeviceComm: Send + Sync + 'static {
    /// Get device features and information
    fn get_features(&self) -> Result<Features>;
    
    /// Get a Bitcoin address for the given path and coin
    fn get_address(&self, coin_name: &str, address_n: &[u32], show_display: bool) -> Result<Address>;
    
    /// Sign a Bitcoin transaction
    fn sign_tx(&self, coin_name: &str, inputs: &[TxInput], outputs: &[TxOutput]) -> Result<SignedTx>;
    
    /// Sign a message
    fn sign_message(&self, coin_name: &str, address_n: &[u32], message: &str) -> Result<SignedMessage>;
    
    /// Verify a message signature
    fn verify_message(&self, coin_name: &str, address: &str, signature: &str, message: &str) -> Result<bool>;
    
    /// Ping the device
    fn ping(&self, message: &str, button_protection: bool) -> Result<String>;
    
    /// Clear session
    fn clear_session(&self) -> Result<()>;
    
    /// Get entropy from device
    fn get_entropy(&self, size: u32) -> Result<Vec<u8>>;
    
    /// Device management: change pin
    fn change_pin(&self, remove: bool) -> Result<()>;
}

/// Mock device communication for testing without hardware
pub struct MockDeviceComm;

impl DeviceComm for MockDeviceComm {
    fn get_features(&self) -> Result<Features> {
        Ok(Features {
            vendor: "KeepKey".to_string(),
            major_version: 7,
            minor_version: 1,
            patch_version: 0,
            bootloader_mode: false,
            device_id: "MOCK_DEVICE_ID".to_string(),
            pin_protection: true,
            passphrase_protection: false,
            label: "Mock KeepKey".to_string(),
            initialized: true,
            revision: "MOCK_REVISION".to_string(),
            bootloader_hash: "MOCK_BOOTLOADER_HASH".to_string(),
            model: "1".to_string(),
            needs_backup: false,
            capabilities: vec!["Bitcoin".to_string()],
            flags: None,
        })
    }
    
    fn get_address(&self, coin_name: &str, address_n: &[u32], _show_display: bool) -> Result<Address> {
        // Mock addresses based on derivation path
        let addr = match coin_name {
            "Bitcoin" => "bc1qmock0000000000000000000000000000qqqmock",
            "Testnet" => "tb1qmock0000000000000000000000000000qqqmock",
            _ => "UNSUPPORTED_COIN",
        };
        
        Ok(Address {
            address: addr.to_string(),
            path: format!("{}", address_n.iter().map(|n| n.to_string()).collect::<Vec<_>>().join("/")),
        })
    }
    
    fn sign_tx(&self, _coin_name: &str, inputs: &[TxInput], outputs: &[TxOutput]) -> Result<SignedTx> {
        // Just return mock signatures for each input
        let signatures = inputs.iter().map(|_| "MOCK_SIGNATURE".to_string()).collect();
        
        Ok(SignedTx {
            signatures,
            serialized_tx: "01000000MOCK_TX_DATA".to_string(),
        })
    }
    
    fn sign_message(&self, _coin_name: &str, _address_n: &[u32], message: &str) -> Result<SignedMessage> {
        Ok(SignedMessage {
            address: "bc1qmock0000000000000000000000000000qqqmock".to_string(),
            signature: format!("MOCK_SIG_{}", message),
        })
    }
    
    fn verify_message(&self, _coin_name: &str, _address: &str, _signature: &str, _message: &str) -> Result<bool> {
        // Mock implementation always verifies successfully
        Ok(true)
    }
    
    fn ping(&self, message: &str, _button_protection: bool) -> Result<String> {
        Ok(format!("PONG: {}", message))
    }
    
    fn clear_session(&self) -> Result<()> {
        Ok(())
    }
    
    fn get_entropy(&self, size: u32) -> Result<Vec<u8>> {
        let mut entropy = Vec::with_capacity(size as usize);
        for i in 0..size {
            entropy.push((i % 256) as u8);
        }
        Ok(entropy)
    }
    
    fn change_pin(&self, _remove: bool) -> Result<()> {
        Ok(())
    }
}

// ===== Data models =====

#[derive(Debug, Serialize, Deserialize)]
pub struct Features {
    pub vendor: String,
    pub major_version: u32,
    pub minor_version: u32,
    pub patch_version: u32,
    pub bootloader_mode: bool,
    pub device_id: String,
    pub pin_protection: bool,
    pub passphrase_protection: bool,
    pub label: String,
    pub initialized: bool,
    pub revision: String,
    pub bootloader_hash: String,
    pub model: String,
    pub needs_backup: bool,
    pub capabilities: Vec<String>,
    pub flags: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    pub address: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    pub address_n: Vec<u32>,
    pub prev_hash: String,
    pub prev_index: u32,
    pub script_type: Option<String>,
    pub amount: u64,
    pub sequence: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutput {
    pub address: Option<String>,
    pub address_n: Option<Vec<u32>>,
    pub script_type: Option<String>,
    pub amount: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedTx {
    pub signatures: Vec<String>,
    pub serialized_tx: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedMessage {
    pub address: String,
    pub signature: String,
}
