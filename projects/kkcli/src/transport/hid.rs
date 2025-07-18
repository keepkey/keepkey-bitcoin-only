use anyhow::{anyhow, Result};
use hidapi::{HidApi, HidDevice};
use std::time::Duration;
use thiserror::Error;
use super::{Transport, ProtocolAdapter};
use crate::messages::{self, Message};
use tracing::{info, debug, error};

const KEEPKEY_VID: u16 = 0x2B24;
const KEEPKEY_PIDS: &[u16] = &[0x0001, 0x0002]; // Legacy and bootloader PIDs
const HID_REPORT_SIZE: usize = 64;
const REPORT_ID: u8 = 0;

#[derive(Debug, Error)]
pub enum HidError {
    #[error("HID error: {0}")]
    HidApi(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Other error: {0}")]
    Other(String),
}

pub struct HidTransport {
    device: HidDevice,
}

impl HidTransport {
    /// Create a new HID transport, optionally for a specific device by serial number
    pub fn new(serial_number: Option<&str>) -> Result<Self> {
        let api = HidApi::new().map_err(|e| anyhow!("Failed to initialize HID API: {}", e))?;
        
        // Find KeepKey devices
        let keepkey_devices: Vec<_> = api.device_list()
            .filter(|info| info.vendor_id() == KEEPKEY_VID && KEEPKEY_PIDS.contains(&info.product_id()))
            .collect();
        
        if keepkey_devices.is_empty() {
            return Err(anyhow!("No KeepKey devices found"));
        }
        
        // Find the requested device or use the first one
        let device = if let Some(serial) = serial_number {
            // Try exact serial match first
            let exact_match = keepkey_devices.iter()
                .find(|info| info.serial_number() == Some(serial));
                    
            if let Some(info) = exact_match {
                info.open_device(&api)
                    .map_err(|e| anyhow!("Failed to open device with serial {}: {}", serial, e))?
            } else {
                // Fall back to first available KeepKey
                eprintln!("Warning: No KeepKey with serial {} found, using first available", serial);
                keepkey_devices[0].open_device(&api)
                    .map_err(|e| anyhow!("Failed to open KeepKey device: {}", e))?
            }
        } else {
            // No serial specified, use first device
            keepkey_devices[0].open_device(&api)
                .map_err(|e| anyhow!("Failed to open KeepKey device: {}", e))?
        };
        
        Ok(Self { device })
    }
}

impl Transport for HidTransport {
    type Error = HidError;
    
    fn write(&mut self, msg: &[u8], _timeout: Duration) -> Result<usize, Self::Error> {
        // The incoming message already has the protocol header from Message::encode
        // Format: [#][#][msg_type(2)][length(4)][data...]
        
        if msg.len() < 8 {
            return Err(HidError::Other("Message too short".to_string()));
        }
        
        // Extract message components (skip the ## header)
        let msg_type = &msg[2..4];
        let msg_length = &msg[4..8];
        let msg_data = &msg[8..];
        
        let msg_type_val = u16::from_be_bytes([msg_type[0], msg_type[1]]);
        let msg_length_val = u32::from_be_bytes([msg_length[0], msg_length[1], msg_length[2], msg_length[3]]);
        
        info!("HID Write: Sending message type {} (0x{:04x}), length: {}", 
              msg_type_val, msg_type_val, msg_length_val);
        
        // Log first few bytes of data for debugging
        if msg_data.len() > 0 {
            let preview_len = msg_data.len().min(32);
            let preview: Vec<String> = msg_data[..preview_len].iter()
                .map(|b| format!("{:02x}", b))
                .collect();
            debug!("HID Write: First {} data bytes: {}", preview_len, preview.join(" "));
        }
        
        // Prepare first packet with v4 format
        let mut first_packet = vec![0u8; HID_REPORT_SIZE];
        first_packet[0] = REPORT_ID;
        first_packet[1] = 0x3f;
        first_packet[2] = 0x23;
        first_packet[3] = 0x23;
        first_packet[4..6].copy_from_slice(msg_type);
        first_packet[6..10].copy_from_slice(msg_length);
        
        // Copy as much data as fits in first packet
        let first_chunk_size = (HID_REPORT_SIZE - 10).min(msg_data.len());
        if first_chunk_size > 0 {
            first_packet[10..10 + first_chunk_size].copy_from_slice(&msg_data[..first_chunk_size]);
        }
        
        debug!("HID Write: Sending first packet (64 bytes), data chunk size: {}", first_chunk_size);
        
        // Send first packet
        self.device
            .write(&first_packet)
            .map_err(|e| HidError::HidApi(format!("HID write failed: {}", e)))?;
        
        // Send continuation packets if needed
        let mut sent = first_chunk_size;
        let mut packet_count = 1;
        while sent < msg_data.len() {
            let mut cont_packet = vec![0u8; HID_REPORT_SIZE];
            cont_packet[0] = b'?'; // Continuation packet marker
            
            let chunk_size = (HID_REPORT_SIZE - 1).min(msg_data.len() - sent);
            cont_packet[1..1 + chunk_size].copy_from_slice(&msg_data[sent..sent + chunk_size]);
            
            debug!("HID Write: Sending continuation packet {} (64 bytes), data chunk size: {}", 
                   packet_count + 1, chunk_size);
            
            self.device
                .write(&cont_packet)
                .map_err(|e| HidError::HidApi(format!("HID continuation write failed: {}", e)))?;
            
            sent += chunk_size;
            packet_count += 1;
        }
        
        info!("HID Write: Complete. Sent {} bytes in {} packets", msg.len(), packet_count);
        
        Ok(msg.len())
    }
    
    fn read(&mut self, buf: &mut Vec<u8>, timeout: Duration) -> Result<(), Self::Error> {
        let timeout_ms = timeout.as_millis() as i32;
        
        info!("HID Read: Waiting for response (timeout: {} ms)...", timeout_ms);
        
        // Read first packet
        let mut packet = vec![0u8; HID_REPORT_SIZE];
        let size = self.device
            .read_timeout(&mut packet, timeout_ms)
            .map_err(|e| HidError::HidApi(format!("HID read failed: {}", e)))?;
        
        if size == 0 {
            error!("HID Read: No data received from device after {} ms timeout", timeout_ms);
            return Err(HidError::Other("No data received from device".to_string()));
        }
        
        info!("HID Read: Received first packet ({} bytes)", size);
        
        // Log first few bytes for debugging
        let preview: Vec<String> = packet[..size.min(16)].iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        debug!("HID Read: First 16 bytes: {}", preview.join(" "));
        
        // v4 response format: [0x3f][0x23][0x23][msg_type(2)][length(4)][data...]
        if size < 9 || packet[0] != 0x3f || packet[1] != 0x23 || packet[2] != 0x23 {
            error!("HID Read: Invalid response header");
            return Err(HidError::Other(format!(
                "Invalid response header: {:02x} {:02x} {:02x}",
                packet[0], packet[1], packet[2]
            )));
        }
        
        // Extract message type and length from header
        let msg_type = u16::from_be_bytes([packet[3], packet[4]]);
        let msg_length = u32::from_be_bytes([packet[5], packet[6], packet[7], packet[8]]) as usize;
        
        info!("HID Read: Message type {} (0x{:04x}), length: {} bytes", 
              msg_type, msg_type, msg_length);
        
        // Convert to v5 format for the protocol adapter
        // Start with ## header that Message::decode expects
        buf.clear();
        buf.push(b'#');
        buf.push(b'#');
        buf.extend_from_slice(&packet[3..9]); // msg_type(2) + length(4)
        
        // Copy initial data from first packet
        let first_chunk_size = (size - 9).min(msg_length);
        buf.extend_from_slice(&packet[9..9 + first_chunk_size]);
        
        let mut bytes_received = first_chunk_size;
        debug!("HID Read: First packet contained {} bytes of message data", first_chunk_size);
        
        // Read continuation packets if needed
        let mut packet_count = 1;
        while bytes_received < msg_length {
            packet.fill(0);
            
            debug!("HID Read: Reading continuation packet {} (need {} more bytes)...", 
                   packet_count + 1, msg_length - bytes_received);
                   
            let cont_size = self.device
                .read_timeout(&mut packet, 100)
                .map_err(|e| HidError::HidApi(format!("HID continuation read failed: {}", e)))?;
            
            if cont_size > 0 {
                // Skip the '?' prefix byte in continuation packets
                let data_start = if packet[0] == b'?' { 1 } else { 0 };
                let remaining = msg_length - bytes_received;
                let to_copy = (cont_size - data_start).min(remaining);
                
                debug!("HID Read: Continuation packet {} has {} bytes, copying {} bytes", 
                       packet_count + 1, cont_size, to_copy);
                
                buf.extend_from_slice(&packet[data_start..data_start + to_copy]);
                bytes_received += to_copy;
                packet_count += 1;
            } else if bytes_received < msg_length {
                // Still need more data but didn't get any
                error!("HID Read: Incomplete message after {} packets", packet_count);
                return Err(HidError::Other(format!(
                    "Incomplete message: expected {} bytes, got {}",
                    msg_length, bytes_received
                )));
            }
        }
        
        info!("HID Read: Complete. Received {} bytes in {} packets", buf.len() - 2, packet_count);
        
        Ok(())
    }
    
    fn reset(&mut self) -> Result<(), Self::Error> {
        // HID doesn't have a direct reset like USB
        // Flush any pending data by reading with a short timeout
        let mut dummy = vec![0u8; HID_REPORT_SIZE];
        while self.device.read_timeout(&mut dummy, 10).unwrap_or(0) > 0 {
            // Keep reading until no more data
        }
        Ok(())
    }
}

// ProtocolAdapter is automatically implemented for types that implement Transport 