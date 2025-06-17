use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;
use anyhow::{anyhow, Result};
use tracing::{info, warn, error, debug, instrument};

use crate::messages::{Message, GetFeatures, GetAddress, Features};
use crate::transport::ProtocolAdapter;
use crate::usb_manager::FriendlyUsbDevice;

// Default timeouts and limits
const DEVICE_OPERATION_TIMEOUT: Duration = Duration::from_secs(30);
const QUEUE_CHANNEL_SIZE: usize = 100;
const CACHE_MAX_ENTRIES: usize = 256;
const CACHE_TTL: Duration = Duration::from_secs(30);

/// Unique key for caching device responses
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    device_id: String,
    operation: String,
    params_hash: u64,
}

impl CacheKey {
    fn new(device_id: String, operation: impl Into<String>, params: &[u8]) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        params.hash(&mut hasher);
        
        Self {
            device_id,
            operation: operation.into(),
            params_hash: hasher.finish(),
        }
    }
}

/// Cached response with timestamp
#[derive(Debug, Clone)]
pub struct CachedResponse {
    value: serde_json::Value,
    timestamp: Instant,
}

impl CachedResponse {
    fn new(value: serde_json::Value) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
        }
    }
    
    fn is_fresh(&self) -> bool {
        self.timestamp.elapsed() < CACHE_TTL
    }
}

/// Commands that can be sent to the device worker
#[derive(Debug)]
pub enum DeviceCmd {
    GetFeatures {
        respond_to: oneshot::Sender<Result<Features>>,
        enqueued_at: Instant,
    },
    GetAddress {
        path: Vec<u32>,
        coin_name: String,
        script_type: Option<i32>,
        respond_to: oneshot::Sender<Result<String>>,
        enqueued_at: Instant,
    },
    SendRaw {
        message: Message,
        respond_to: oneshot::Sender<Result<Message>>,
        enqueued_at: Instant,
        bypass_cache: bool,
    },
    Shutdown {
        respond_to: oneshot::Sender<Result<()>>,
    },
}

impl DeviceCmd {
    fn enqueued_at(&self) -> Instant {
        match self {
            DeviceCmd::GetFeatures { enqueued_at, .. } => *enqueued_at,
            DeviceCmd::GetAddress { enqueued_at, .. } => *enqueued_at,
            DeviceCmd::SendRaw { enqueued_at, .. } => *enqueued_at,
            DeviceCmd::Shutdown { .. } => Instant::now(),
        }
    }
    
    fn operation_name(&self) -> &'static str {
        match self {
            DeviceCmd::GetFeatures { .. } => "get_features",
            DeviceCmd::GetAddress { .. } => "get_address", 
            DeviceCmd::SendRaw { .. } => "send_raw",
            DeviceCmd::Shutdown { .. } => "shutdown",
        }
    }
    
    fn should_cache(&self) -> bool {
        match self {
            DeviceCmd::GetFeatures { .. } => true,
            DeviceCmd::GetAddress { .. } => true,
            DeviceCmd::SendRaw { bypass_cache, .. } => !*bypass_cache,
            DeviceCmd::Shutdown { .. } => false,
        }
    }
}

/// Metrics for monitoring queue performance
#[derive(Debug, Default, Clone)]
pub struct DeviceQueueMetrics {
    pub queue_wait_ms: Vec<u64>,
    pub device_rtt_ms: Vec<u64>,
    pub total_ms: Vec<u64>,
    pub queue_depth: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl DeviceQueueMetrics {
    pub fn cache_hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
    
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }
    
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }
    
    pub fn record_operation(&mut self, queue_wait: Duration, device_rtt: Duration, total: Duration) {
        self.queue_wait_ms.push(queue_wait.as_millis() as u64);
        self.device_rtt_ms.push(device_rtt.as_millis() as u64);
        self.total_ms.push(total.as_millis() as u64);
        
        // Keep only last 100 measurements
        if self.queue_wait_ms.len() > 100 {
            self.queue_wait_ms.remove(0);
            self.device_rtt_ms.remove(0);
            self.total_ms.remove(0);
        }
    }
}

/// Worker task that processes device commands sequentially
pub struct DeviceWorker {
    device_id: String,
    device_info: FriendlyUsbDevice,
    transport: Option<Box<dyn ProtocolAdapter + Send>>,
    cache: HashMap<CacheKey, CachedResponse>,
    metrics: DeviceQueueMetrics,
    cmd_rx: mpsc::Receiver<DeviceCmd>,
}

impl DeviceWorker {
    fn new(
        device_id: String,
        device_info: FriendlyUsbDevice,
        cmd_rx: mpsc::Receiver<DeviceCmd>,
    ) -> Self {
        Self {
            device_id,
            device_info,
            transport: None,
            cache: HashMap::new(),
            metrics: DeviceQueueMetrics::default(),
            cmd_rx,
        }
    }
    
    /// Main worker loop - processes commands sequentially
    #[instrument(level = "info", skip(self))]
    pub async fn run(mut self) {
        info!("🚀 DeviceWorker starting for device {}", self.device_id);
        
        while let Some(cmd) = self.cmd_rx.recv().await {
            let start_time = Instant::now();
            let queue_wait = start_time.duration_since(cmd.enqueued_at());
            
            // Update queue depth metric
            self.metrics.queue_depth = self.cmd_rx.len();
            
            debug!("📝 Processing {} command (queue wait: {:?})", cmd.operation_name(), queue_wait);
            
            let result = self.process_command(cmd).await;
            
            if let Err(ref e) = result {
                error!("❌ Command failed: {}", e);
            }
        }
        
        info!("🛑 DeviceWorker shutting down for device {}", self.device_id);
    }
    
    /// Process a single command
    async fn process_command(&mut self, cmd: DeviceCmd) -> Result<()> {
        let device_start = Instant::now();
        let enqueued_at = cmd.enqueued_at();
        
        match cmd {
            DeviceCmd::GetFeatures { respond_to, .. } => {
                let result = self.handle_get_features().await;
                let _ = respond_to.send(result);
            }
            DeviceCmd::GetAddress { path, coin_name, script_type, respond_to, .. } => {
                let result = self.handle_get_address(path, coin_name, script_type).await;
                let _ = respond_to.send(result);
            }
            DeviceCmd::SendRaw { message, respond_to, bypass_cache, .. } => {
                let result = self.handle_send_raw(message, bypass_cache).await;
                let _ = respond_to.send(result);
            }
            DeviceCmd::Shutdown { respond_to } => {
                let _ = respond_to.send(Ok(()));
                return Ok(());
            }
        }
        
        let device_rtt = device_start.elapsed();
        let total_time = enqueued_at.elapsed();
        let queue_wait = device_start.duration_since(enqueued_at);
        
        self.metrics.record_operation(queue_wait, device_rtt, total_time);
        
        Ok(())
    }
    
    /// Create transport with USB/HID fallback
    fn create_transport_for_device(device_info: &FriendlyUsbDevice) -> Result<Box<dyn ProtocolAdapter + Send>> {
        // Find physical device for transport
        let devices = crate::features::list_devices();
        let physical_device = Self::find_physical_device_by_info(device_info, &devices)?;
        
        // Try USB transport first
        match crate::transport::UsbTransport::new(&physical_device, 0) {
            Ok((transport, _, _)) => {
                info!("✅ Created USB transport for device {}", device_info.unique_id);
                Ok(Box::new(transport))
            }
            Err(usb_err) => {
                warn!("⚠️ USB transport failed for device {}: {}, trying HID fallback", device_info.unique_id, usb_err);
                
                // Try HID fallback
                match crate::transport::HidTransport::new_for_device(device_info.serial_number.as_deref()) {
                    Ok(hid_transport) => {
                        info!("✅ Created HID transport for device {}", device_info.unique_id);
                        Ok(Box::new(hid_transport))
                    }
                    Err(hid_err) => {
                        Err(anyhow!("Failed with both USB ({}) and HID ({})", usb_err, hid_err))
                    }
                }
            }
        }
    }
    
    /// Find the physical device matching device info (static method)
    fn find_physical_device_by_info(device_info: &FriendlyUsbDevice, devices: &[rusb::Device<rusb::GlobalContext>]) -> Result<rusb::Device<rusb::GlobalContext>> {
        if let Some(serial) = &device_info.serial_number {
            // Match by serial number
            for device in devices {
                if let Ok(handle) = device.open() {
                    let timeout = std::time::Duration::from_millis(100);
                    if let Ok(langs) = handle.read_languages(timeout) {
                        if let Some(lang) = langs.first() {
                            if let Ok(desc) = device.device_descriptor() {
                                if let Ok(device_serial) = handle.read_serial_number_string(*lang, &desc, timeout) {
                                    if device_serial == *serial {
                                        return Ok(device.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Try to parse bus and address from unique_id
        let parts: Vec<&str> = device_info.unique_id.split('_').collect();
        if parts.len() >= 2 {
            let bus_str = parts[0].strip_prefix("bus").unwrap_or("");
            let addr_str = parts[1].strip_prefix("addr").unwrap_or("");
            
            if let (Ok(bus), Ok(addr)) = (bus_str.parse::<u8>(), addr_str.parse::<u8>()) {
                for device in devices {
                    if device.bus_number() == bus && device.address() == addr {
                        return Ok(device.clone());
                    }
                }
            }
        }
        
        Err(anyhow!("Physical device not found for {}", device_info.unique_id))
    }
    
    /// Ensure transport is available, creating if necessary
    async fn ensure_transport(&mut self) -> Result<&mut (dyn ProtocolAdapter + Send)> {
        if self.transport.is_none() {
            info!("🔗 Creating transport for device {}", self.device_id);
            // Use static method to avoid &self borrow
            self.transport = Some(Self::create_transport_for_device(&self.device_info)?);
        }
        
        Ok(self.transport.as_mut().unwrap().as_mut())
    }
    
    /// Handle GetFeatures command with caching
    async fn handle_get_features(&mut self) -> Result<Features> {
        // Skip caching for Features due to serde compatibility issues
        // Features are lightweight and we want fresh data anyway
        self.metrics.record_cache_miss();
        
        // Execute on device
        let transport = self.ensure_transport().await?;
        let response = transport.with_standard_handler().handle(GetFeatures {}.into())?;
        
        match response {
            Message::Features(features) => Ok(features),
            _ => Err(anyhow!("Unexpected response to GetFeatures")),
        }
    }
    
    /// Handle GetAddress command with caching
    async fn handle_get_address(&mut self, path: Vec<u32>, coin_name: String, script_type: Option<i32>) -> Result<String> {
        // Simple hash for parameters without bincode dependency
        let mut params = Vec::new();
        for &part in &path {
            params.extend_from_slice(&part.to_le_bytes());
        }
        params.extend_from_slice(coin_name.as_bytes());
        if let Some(st) = script_type {
            params.extend_from_slice(&st.to_le_bytes());
        }
        
        let cache_key = CacheKey::new(self.device_id.clone(), "get_address", &params);
        
        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key) {
            if cached.is_fresh() {
                self.metrics.record_cache_hit();
                debug!("💰 Cache hit for GetAddress");
                return Ok(cached.value.as_str().unwrap_or_default().to_string());
            }
        }
        
        self.metrics.record_cache_miss();
        
        // Execute on device
        let transport = self.ensure_transport().await?;
        let get_address = GetAddress {
            address_n: path,
            coin_name: Some(coin_name),
            script_type,
            ..Default::default()
        };
        
        let response = transport.with_standard_handler().handle(get_address.into())?;
        
        match response {
            Message::Address(addr_response) => {
                let address = addr_response.address.clone(); // Use field directly not method
                
                // Cache the response
                if let Ok(json_value) = serde_json::to_value(&address) {
                    self.cache.insert(cache_key, CachedResponse::new(json_value));
                    self.cleanup_cache();
                }
                
                Ok(address)
            }
            _ => Err(anyhow!("Unexpected response to GetAddress")),
        }
    }
    
    /// Handle raw message sending 
    async fn handle_send_raw(&mut self, message: Message, bypass_cache: bool) -> Result<Message> {
        // For raw messages, we generally don't cache unless specifically allowed
        let transport = self.ensure_transport().await?;
        let response = transport.with_standard_handler().handle(message)?;
        
        // If this was a mutable operation, purge cache
        if bypass_cache || self.is_mutable_operation(&response) {
            self.cache.clear();
            info!("🧹 Cache cleared due to mutable operation");
        }
        
        Ok(response)
    }
    
    /// Check if an operation is mutable and should invalidate cache
    fn is_mutable_operation(&self, response: &Message) -> bool {
        matches!(response,
            Message::Success(_) | // Most success responses indicate state change
            Message::TxRequest(_) | // Transaction operations
            Message::PinMatrixRequest(_) | // PIN operations
            Message::PassphraseRequest(_) // Passphrase operations
        )
    }
    
    /// Clean up old cache entries
    fn cleanup_cache(&mut self) {
        // Remove expired entries
        self.cache.retain(|_, cached| cached.is_fresh());
        
        // If still too many entries, remove oldest
        if self.cache.len() > CACHE_MAX_ENTRIES {
            let keys_to_remove: Vec<_> = {
                let mut entries: Vec<_> = self.cache.iter().collect();
                entries.sort_by_key(|(_, cached)| cached.timestamp);
                
                let to_remove = entries.len() - CACHE_MAX_ENTRIES;
                entries.iter().take(to_remove).map(|(key, _)| (*key).clone()).collect()
            };
            
            for key in keys_to_remove {
                self.cache.remove(&key);
            }
        }
    }
}

/// Handle for communicating with a device worker
#[derive(Clone, Debug)]
pub struct DeviceQueueHandle {
    device_id: String,
    cmd_tx: mpsc::Sender<DeviceCmd>,
}

impl DeviceQueueHandle {
    pub fn new(device_id: String, cmd_tx: mpsc::Sender<DeviceCmd>) -> Self {
        Self { device_id, cmd_tx }
    }
    
    /// Get device features
    #[instrument(level = "debug", skip(self))]
    pub async fn get_features(&self) -> Result<Features> {
        let (tx, rx) = oneshot::channel();
        let cmd = DeviceCmd::GetFeatures {
            respond_to: tx,
            enqueued_at: Instant::now(),
        };
        
        self.cmd_tx.send(cmd).await
            .map_err(|_| anyhow!("Device worker unavailable"))?;
            
        timeout(DEVICE_OPERATION_TIMEOUT, rx).await
            .map_err(|_| anyhow!("Device operation timed out"))?
            .map_err(|_| anyhow!("Device worker channel closed"))?
    }
    
    /// Get address for given path
    #[instrument(level = "debug", skip(self))]
    pub async fn get_address(&self, path: Vec<u32>, coin_name: String, script_type: Option<i32>) -> Result<String> {
        let (tx, rx) = oneshot::channel();
        let cmd = DeviceCmd::GetAddress {
            path,
            coin_name,
            script_type,
            respond_to: tx,
            enqueued_at: Instant::now(),
        };
        
        self.cmd_tx.send(cmd).await
            .map_err(|_| anyhow!("Device worker unavailable"))?;
            
        timeout(DEVICE_OPERATION_TIMEOUT, rx).await
            .map_err(|_| anyhow!("Device operation timed out"))?
            .map_err(|_| anyhow!("Device worker channel closed"))?
    }
    
    /// Send raw message to device
    #[instrument(level = "debug", skip(self, message))]
    pub async fn send_raw(&self, message: Message, bypass_cache: bool) -> Result<Message> {
        let (tx, rx) = oneshot::channel();
        let cmd = DeviceCmd::SendRaw {
            message,
            respond_to: tx,
            enqueued_at: Instant::now(),
            bypass_cache,
        };
        
        self.cmd_tx.send(cmd).await
            .map_err(|_| anyhow!("Device worker unavailable"))?;
            
        timeout(DEVICE_OPERATION_TIMEOUT, rx).await
            .map_err(|_| anyhow!("Device operation timed out"))?
            .map_err(|_| anyhow!("Device worker channel closed"))?
    }
    
    /// Shutdown the device worker
    pub async fn shutdown(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        let cmd = DeviceCmd::Shutdown { respond_to: tx };
        
        self.cmd_tx.send(cmd).await
            .map_err(|_| anyhow!("Device worker unavailable"))?;
            
        timeout(Duration::from_secs(5), rx).await
            .map_err(|_| anyhow!("Shutdown timed out"))?
            .map_err(|_| anyhow!("Device worker channel closed"))?
    }
    
    pub fn device_id(&self) -> &str {
        &self.device_id
    }
}

/// Factory for creating device workers and handles
pub struct DeviceQueueFactory;

impl DeviceQueueFactory {
    /// Spawn a new device worker and return a handle to it
    pub fn spawn_worker(device_id: String, device_info: FriendlyUsbDevice) -> DeviceQueueHandle {
        let (cmd_tx, cmd_rx) = mpsc::channel(QUEUE_CHANNEL_SIZE);
        
        let worker = DeviceWorker::new(device_id.clone(), device_info, cmd_rx);
        
        // Spawn the worker task
        tokio::spawn(worker.run());
        
        DeviceQueueHandle::new(device_id, cmd_tx)
    }
} 