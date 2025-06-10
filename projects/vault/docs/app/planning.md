Below is a “single-controller” blueprint that plugs straight into the code you posted and lets Tauri commands, REST routes, and MCP JSON-RPC all talk to the same device state without ever opening USB twice.

1. Core idea
   php
   Copy
   Edit
   ┌─────────────────────────┐
   │   DeviceController      │   ← the ONLY place that touches rusb/hidapi
   │  (owns rusb::Context)   │
   ├─────────────────────────┤
   │ • HashMap<String,DeviceEntry>   (RwLock) ─ current snapshot
   │ • broadcast::Sender<DeviceEvent>          ─ fire-and-forget events
   │ • fn start_hotplug_thread()               ─ handles Context::handle_events
   │ • fn list_devices() -> Vec<FriendlyUsbDevice>
   │ • fn get_features(id) -> Option<DeviceFeatures>
   │ • … any other helpers
   └────────────┬────────────┘
   │  Arc<DeviceController>  (global)
   ┌─────────────┴─────────────┐
   │                           │
   ┌────────▼────────┐        ┌─────────▼─────────┐
   │  Tauri commands │        │  Axum REST / MCP │
   │  (.manage())    │        │ .with_state(...)  │
   └─────────────────┘        └───────────────────┘
   ╲
   ╲   broadcast::Receiver<DeviceEvent>
   ╲   –> maps to
   ╲     • app.emit("usb-device-*")
   ╲    • SSE stream /ws if you want
   Only one rusb::Context (and thus one OS USB handle) exists.
   Everyone else just borrows read-only snapshots or subscribes to the event bus.

2. DeviceController skeleton
   rust
   Copy
   Edit
   // src-tauri/src/device_controller.rs
   use std::{
   collections::HashMap,
   sync::Arc,
   };
   use tokio::sync::{broadcast, RwLock};
   use once_cell::sync::Lazy;
   use anyhow::Result;
   use crate::{
   features::{self, DeviceFeatures},
   usb_manager::{RawUsbDevice, FriendlyUsbDevice, KEEPKEY_VID},
   };

pub struct DeviceEntry {
pub device: FriendlyUsbDevice,
pub features: Option<DeviceFeatures>,
pub last_seen: std::time::SystemTime,
}

pub struct DeviceController {
ctx: rusb::Context,                    // lives only on the hotplug thread
devices: RwLock<HashMap<String,DeviceEntry>>,
tx: broadcast::Sender<DeviceEvent>,    // multi-consumer event bus
}

#[derive(Clone, Debug)]
pub struct DeviceEvent {
pub kind: EventKind,
pub device: FriendlyUsbDevice,
pub features: Option<DeviceFeatures>,
}
#[derive(Clone, Debug)]
pub enum EventKind { Connected, Disconnected, Updated }

impl DeviceController {
fn new() -> Result<Arc<Self>> {
let (tx, _) = broadcast::channel(32);
Ok(Arc::new(Self{
ctx: rusb::Context::new()?,
devices: RwLock::new(HashMap::new()),
tx,
}))
}

    /// Fire-and-forget thread. Owns rusb::Context forever.
    fn start_hotplug_thread(self: &Arc<Self>) {
        let this = Arc::clone(self);
        std::thread::spawn(move || {
            // register hotplug…
            // on change -> this.handle_arrival(...), this.handle_departure(...)
            loop {
                let _ = this.ctx.handle_events(None);
            }
        });
    }

    fn handle_arrival(&self, raw: RawUsbDevice) {
        let device = FriendlyUsbDevice::from_raw(&raw);
        let features = if device.is_keepkey {
            features::get_device_features_for_device(&device).ok()
        } else { None };

        {
            let mut map = futures::executor::block_on(self.devices.write());
            map.insert(device.unique_id.clone(), DeviceEntry {
                device: device.clone(),
                features: features.clone(),
                last_seen: std::time::SystemTime::now(),
            });
        }
        let _ = self.tx.send(DeviceEvent{ kind: EventKind::Connected, device, features });
    }

    fn handle_departure(&self, unique_id: String) {
        if let Some(entry) = futures::executor::block_on(self.devices.write()).remove(&unique_id) {
            let _ = self.tx.send(DeviceEvent{
                kind: EventKind::Disconnected,
                device: entry.device,
                features: entry.features,
            });
        }
    }

    /* ---------- public API used by routes/commands ---------- */

    pub async fn list_devices(&self) -> Vec<FriendlyUsbDevice> {
        self.devices.read().await.values().map(|e| e.device.clone()).collect()
    }

    pub async fn get_features(&self, id: &str) -> Option<DeviceFeatures> {
        self.devices.read().await.get(id).and_then(|e| e.features.clone())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DeviceEvent> {
        self.tx.subscribe()
    }
}

/* ---------- global accessor ---------- */

static GLOBAL_DC: Lazy<Arc<DeviceController>> = Lazy::new(|| {
let dc = DeviceController::new().expect("USB init failed");
dc.start_hotplug_thread();
dc
});

pub fn global() -> Arc<DeviceController> { Arc::clone(&GLOBAL_DC) }
3. Wiring it into Tauri
   rust
   Copy
   Edit
   // in lib.rs setup()
   let dc = device_controller::global();

// put into Tauri state so |#[tauri::command]| can reach it
app.manage(dc.clone());

// forward hotplug notifications to the front-end
let mut recv = dc.subscribe();
tauri::async_runtime::spawn(async move {
while let Ok(evt) = recv.recv().await {
let _ = app.emit_all("usb:event", &evt);
}
});
Now any command can do:

rust
Copy
Edit
#[tauri::command]
async fn list_usb_devices(state: tauri::State<'_, Arc<DeviceController>>)
-> Vec<FriendlyUsbDevice>
{
state.list_devices().await
}
4. Wiring it into Axum REST + MCP
   rust
   Copy
   Edit
   // server/start_server()
   let dc = device_controller::global();
   let server_state = Arc::new(ServerState { dc });

// Axum boilerplate
Router::new()
.route("/api/devices", get(routes::list_devices))
.with_state(server_state.clone())
...
Routes now just borrow:

rust
Copy
Edit
pub async fn list_devices(
State(st): State<Arc<ServerState>>,
) -> Result<Json<Vec<UsbDeviceInfo>>, StatusCode> {
let list = st.dc.list_devices().await;
Ok(Json(list.into_iter().map(|d| UsbDeviceInfo::from(d)).collect()))
}
Want SSE? Convert a broadcast::Receiver into a stream:

rust
Copy
Edit
pub async fn mcp_sse_handler(
State(st): State<Arc<ServerState>>,
) -> Sse<impl Stream<Item = Result<Event,Infallible>>> {
let mut rx = st.dc.subscribe();
Sse::new(tokio_stream::wrappers::BroadcastStream::new(rx)
.filter_map(|e| async move {
e.ok().map(|evt| Ok(Event::default().data(serde_json::to_string(&evt).unwrap())))
}))
}
5. Deleting the old duplicates
   usb_manager::init_device_manager ➜ becomes internal helper used only by DeviceController

The global DEVICE_FEATURES Lazy<Mutex<_>> is no longer needed; query the controller instead.

spawn_usb_watcher collapses into the single broadcast subscription shown above.

Axum’s ServerState no longer keeps its own DeviceManager; it just holds Arc<DeviceController>.

6. Migration checklist
   Step	Action
   1	Add device_controller.rs (above)
   2	Replace every call to usb_manager::init_device_manager outside the controller with device_controller::global()
   3	Remove the second DeviceManager created for the server in lib.rs
   4	Delete DEVICE_FEATURES global and refactor features::get_device_features_impl() to call device_controller::global().get_features(first_id).await
   5	Update all Tauri commands, REST handlers, and MCP tools to take an Arc<DeviceController> from state instead of spinning up their own manager
   6	Expose events: front-end listens on "usb:event"; REST/MCP can expose SSE if desired
   7	Thorough test: plug/unplug multiple KeepKeys, verify one arrival message, no duplicate device openings, and routes all show identical lists

7. Benefits recap
   Zero USB contention – only one rusb context.

O(1) feature fetches from an in-memory snapshot, no re-scan per request.

Unified events – every layer receives identical DeviceEvents.

Extensible – want Bluetooth or WebHID later? Add another backend inside the controller without changing any API boundaries.

Drop this controller in, rip out the extra managers, and you’ll have a single source of truth for KeepKey (or any USB device) across your whole Tauri stack.