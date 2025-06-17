use tauri::State;
use uuid::Uuid;

use crate::device_queue::{DeviceQueueManager, DeviceRequest, DeviceRequestWrapper, QueueStatus};

#[tauri::command]
pub async fn add_to_device_queue(
    device_id: String,
    request: DeviceRequest,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<String, String> {
    let request_id = Uuid::new_v4().to_string();
    
    let request_wrapper = DeviceRequestWrapper {
        device_id: device_id.clone(),
        request_id: request_id.clone(),
        request,
    };

    let handle = queue_manager.get_or_create_queue(device_id);
    handle.add_request(request_wrapper).await?;

    Ok(request_id)
}

#[tauri::command]
pub async fn get_queue_status(
    device_id: String,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<QueueStatus, String> {
    let handle = queue_manager.get_or_create_queue(device_id);
    handle.get_status().await
} 