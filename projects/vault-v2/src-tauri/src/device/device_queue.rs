use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::sleep;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceRequest {
    GetXpub {
        path: String,
    },
    GetAddress {
        path: String,
        coin_name: String,
        script_type: Option<String>,
        show_display: Option<bool>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRequestWrapper {
    pub device_id: String,
    pub request_id: String,
    pub request: DeviceRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceResponse {
    Xpub {
        request_id: String,
        device_id: String,
        path: String,
        xpub: String,
        success: bool,
        error: Option<String>,
    },
    Address {
        request_id: String,
        device_id: String,
        path: String,
        address: String,
        success: bool,
        error: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    pub device_id: String,
    pub queue_length: usize,
    pub processing: bool,
    pub last_response: Option<DeviceResponse>,
}

pub enum QueueCommand {
    AddRequest {
        request: DeviceRequestWrapper,
        respond_to: oneshot::Sender<String>,
    },
    GetStatus {
        respond_to: oneshot::Sender<QueueStatus>,
    },
}

pub struct MockDeviceQueue {
    device_id: String,
    cmd_rx: mpsc::Receiver<QueueCommand>,
    queue: Vec<DeviceRequestWrapper>,
    processing: bool,
    last_response: Option<DeviceResponse>,
}

impl MockDeviceQueue {
    fn new(device_id: String, cmd_rx: mpsc::Receiver<QueueCommand>) -> Self {
        Self {
            device_id,
            cmd_rx,
            queue: Vec::new(),
            processing: false,
            last_response: None,
        }
    }

    pub async fn run(mut self) {
        println!("🚀 MockDeviceQueue starting for device {}", self.device_id);

        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                QueueCommand::AddRequest { request, respond_to } => {
                    let request_id = request.request_id.clone();
                    self.queue.push(request);
                    let _ = respond_to.send(request_id);

                    // Process queue if not already processing
                    if !self.processing {
                        self.process_queue().await;
                    }
                }
                QueueCommand::GetStatus { respond_to } => {
                    let status = QueueStatus {
                        device_id: self.device_id.clone(),
                        queue_length: self.queue.len(),
                        processing: self.processing,
                        last_response: self.last_response.clone(),
                    };
                    let _ = respond_to.send(status);
                }
            }
        }

        println!("🛑 MockDeviceQueue shutting down for device {}", self.device_id);
    }

    async fn process_queue(&mut self) {
        self.processing = true;

        while let Some(request_wrapper) = self.queue.pop() {
            match &request_wrapper.request {
                DeviceRequest::GetXpub { path } => {
                    println!("📝 Processing xpub request for device {} path {}",
                        request_wrapper.device_id, path);

                    // Mock 1 second device communication delay
                    sleep(Duration::from_secs(1)).await;

                    let response = self.generate_mock_xpub_response(&request_wrapper, path);
                    println!("✅ Completed xpub request {} -> xpub", request_wrapper.request_id);

                    self.last_response = Some(response);
                }
                DeviceRequest::GetAddress { path, coin_name: _, script_type, show_display } => {
                    let show_on_device = show_display.unwrap_or(false);
                    println!("🏠 Processing address request for device {} path {} (show_display: {})",
                        request_wrapper.device_id, path, show_on_device);

                    // Mock device display confirmation delay (longer for show_display)
                    let delay = if show_on_device {
                        Duration::from_secs(3) // Simulate user confirmation on device
                    } else {
                        Duration::from_secs(1)
                    };
                    sleep(delay).await;

                    let response = self.generate_mock_address_response(&request_wrapper, path, script_type.as_deref());
                    println!("✅ Completed address request {} -> address", request_wrapper.request_id);

                    self.last_response = Some(response);
                }
            }
        }

        self.processing = false;
    }

    fn generate_mock_xpub_response(&self, request: &DeviceRequestWrapper, path: &str) -> DeviceResponse {
        // Hard-coded fake device responses based on derivation path
        let xpub = match path {
            "m/44'/0'/0'" => "xpub6BxKtd6aAuz23XqtWXeSqxShJZn8yqiUmaTdvsPWS3riKkNRcXEPmn1CXmKM1M43mrWfN5QwjdLRghZLrgwMLCeRZqZNuYhVNXr6Pp7aDsH",
            "m/49'/0'/0'" => "ypub6WamSeXgTYgy7W25fVorMLDHFx5SPkuYaE7ToWCiyCUK2jdWpufQ8VqkDg83YjBtJFHDoekhf9ESdPDbL9aCPXC5NnmzXUiq3J6oycFShfS",
            "m/84'/0'/0'" => "zpub6rm1EEJg4JasiTqacdouiUVncAc5ymhKReiPZfLTGnH2GSZquRn9reJhj6sfs73PoSJNXzpERKPVLYbwwUGHNF6jkMX5R58vWaLB9FVyJuX",
            _ => "xpub6BxKtd6aAuz23XqtWXeSqxShJZn8yqiUmaTdvsPWS3riKkNRcXEPmn1CXmKM1M43mrWfN5QwjdLRghZLrgwMLCeRZqZNuYhVNXr6Pp7aDsH" // default
        };

        DeviceResponse::Xpub {
            request_id: request.request_id.clone(),
            device_id: request.device_id.clone(),
            path: path.to_string(),
            xpub: xpub.to_string(),
            success: true,
            error: None,
        }
    }

    fn generate_mock_address_response(&self, request: &DeviceRequestWrapper, path: &str, script_type: Option<&str>) -> DeviceResponse {
        // Generate mock Bitcoin addresses based on path and script type
        let address = match (path, script_type) {
            ("m/44'/0'/0'/0/0", Some("p2pkh")) | ("m/44'/0'/0'/0/0", None) =>
                "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
            ("m/49'/0'/0'/0/0", Some("p2sh-p2wpkh")) =>
                "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy",
            ("m/84'/0'/0'/0/0", Some("p2wpkh")) =>
                "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
            _ => "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2" // default legacy address
        };

        DeviceResponse::Address {
            request_id: request.request_id.clone(),
            device_id: request.device_id.clone(),
            path: path.to_string(),
            address: address.to_string(),
            success: true,
            error: None,
        }
    }
}

#[derive(Clone)]
pub struct DeviceQueueHandle {
    device_id: String,
    cmd_tx: mpsc::Sender<QueueCommand>,
}

impl DeviceQueueHandle {
    pub fn new(device_id: String, cmd_tx: mpsc::Sender<QueueCommand>) -> Self {
        Self { device_id, cmd_tx }
    }

    pub async fn add_request(&self, request: DeviceRequestWrapper) -> Result<String, String> {
        let (tx, rx) = oneshot::channel();
        let cmd = QueueCommand::AddRequest {
            request,
            respond_to: tx,
        };

        self.cmd_tx.send(cmd).await
            .map_err(|_| "Device queue unavailable".to_string())?;

        rx.await.map_err(|_| "Failed to get request ID".to_string())
    }

    pub async fn get_status(&self) -> Result<QueueStatus, String> {
        let (tx, rx) = oneshot::channel();
        let cmd = QueueCommand::GetStatus { respond_to: tx };

        self.cmd_tx.send(cmd).await
            .map_err(|_| "Device queue unavailable".to_string())?;

        rx.await.map_err(|_| "Failed to get status".to_string())
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }
}

pub struct DeviceQueueManager {
    queues: Arc<Mutex<HashMap<String, DeviceQueueHandle>>>,
}

impl DeviceQueueManager {
    pub fn new() -> Self {
        Self {
            queues: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_or_create_queue(&self, device_id: String) -> DeviceQueueHandle {
        let mut queues = self.queues.lock().unwrap();

        if let Some(handle) = queues.get(&device_id) {
            return handle.clone();
        }

        // Create new queue for this device
        let (cmd_tx, cmd_rx) = mpsc::channel(100);
        let handle = DeviceQueueHandle::new(device_id.clone(), cmd_tx);

        // Spawn the device queue worker
        let queue = MockDeviceQueue::new(device_id.clone(), cmd_rx);
        tokio::spawn(async move {
            queue.run().await;
        });

        queues.insert(device_id, handle.clone());
        handle
    }

    pub fn get_all_queue_statuses(&self) -> Vec<String> {
        let queues = self.queues.lock().unwrap();
        queues.keys().cloned().collect()
    }
}