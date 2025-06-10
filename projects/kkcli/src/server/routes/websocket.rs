use axum::{
    extract::{State, WebSocketUpgrade, ws::{WebSocket, Message}},
    response::Response,
};
use std::sync::Arc;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::Serialize;
use serde_json::json;
use tracing::{info, error};
use tokio::time::{interval, Duration};
use tokio::sync::mpsc;

use crate::server::ServerState;

#[derive(Serialize)]
struct DeviceEvent {
    #[serde(rename = "type")]
    event_type: String,
    data: serde_json::Value,
}

// WebSocket handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, _state: Arc<ServerState>) {
    let (mut sender, mut receiver) = socket.split();
    
    info!("WebSocket connection established");
    
    // Create a channel for outgoing messages
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    
    // Send initial connection event
    let connect_event = DeviceEvent {
        event_type: "connected".to_string(),
        data: json!({
            "status": "connected",
            "version": "0.2.3"
        }),
    };
    
    if let Err(e) = tx.send(Message::Text(
        serde_json::to_string(&connect_event).unwrap()
    )) {
        error!("Failed to queue connection event: {}", e);
        return;
    }
    
    // Spawn task to send periodic device status updates
    let status_tx = tx.clone();
    let status_task = tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(5));
        
        loop {
            ticker.tick().await;
            
            // Get device status
            let status = match crate::server::get_device_status_impl().await {
                Ok(status) => status,
                Err(e) => {
                    error!("Failed to get device status: {}", e);
                    continue;
                }
            };
            
            let event = DeviceEvent {
                event_type: "device_status".to_string(),
                data: json!(status),
            };
            
            if let Err(e) = status_tx.send(Message::Text(
                serde_json::to_string(&event).unwrap()
            )) {
                error!("Failed to queue status update: {}", e);
                break;
            }
        }
    });
    
    // Spawn task to forward messages from channel to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = sender.send(msg).await {
                error!("Failed to send WebSocket message: {}", e);
                break;
            }
        }
    });
    
    // Handle incoming messages
    loop {
        tokio::select! {
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        info!("Received WebSocket message: {}", text);
                        
                        // Parse and handle commands
                        if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&text) {
                            handle_ws_command(&tx, cmd).await;
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("WebSocket connection closed");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }
            _ = &mut send_task => {
                // Send task ended
                break;
            }
        }
    }
    
    // Clean up
    status_task.abort();
    info!("WebSocket handler terminated");
}

async fn handle_ws_command(tx: &mpsc::UnboundedSender<Message>, cmd: serde_json::Value) {
    let command = cmd.get("command").and_then(|c| c.as_str()).unwrap_or("");
    
    match command {
        "ping" => {
            let event = DeviceEvent {
                event_type: "pong".to_string(),
                data: json!({ "timestamp": chrono::Utc::now().to_rfc3339() }),
            };
            
            if let Err(e) = tx.send(Message::Text(
                serde_json::to_string(&event).unwrap()
            )) {
                error!("Failed to queue pong: {}", e);
            }
        }
        "get_features" => {
            match crate::server::get_device_features_impl().await {
                Ok(features) => {
                    let event = DeviceEvent {
                        event_type: "features".to_string(),
                        data: serde_json::to_value(features).unwrap(),
                    };
                    
                    if let Err(e) = tx.send(Message::Text(
                        serde_json::to_string(&event).unwrap()
                    )) {
                        error!("Failed to queue features: {}", e);
                    }
                }
                Err(e) => {
                    let event = DeviceEvent {
                        event_type: "error".to_string(),
                        data: json!({ "message": e.to_string() }),
                    };
                    
                    if let Err(e) = tx.send(Message::Text(
                        serde_json::to_string(&event).unwrap()
                    )) {
                        error!("Failed to queue error: {}", e);
                    }
                }
            }
        }
        _ => {
            let event = DeviceEvent {
                event_type: "error".to_string(),
                data: json!({ "message": format!("Unknown command: {}", command) }),
            };
            
            if let Err(e) = tx.send(Message::Text(
                serde_json::to_string(&event).unwrap()
            )) {
                error!("Failed to queue error: {}", e);
            }
        }
    }
} 