use tauri::State;
use uuid::Uuid;

use crate::device_queue::{DeviceQueueManager, XpubRequest, QueueStatus};

#[tauri::command]
pub async fn add_to_device_queue(
    device_id: String,
    path: String,
    queue_manager: State<'_, DeviceQueueManager>,
) -> Result<String, String> {
    let request_id = Uuid::new_v4().to_string();
    
    let request = XpubRequest {
        device_id: device_id.clone(),
        path,
        request_id: request_id.clone(),
    };

    let handle = queue_manager.get_or_create_queue(device_id);
    handle.add_xpub_request(request).await?;

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