Below is a focused code-review aimed at the specific outcome you described: one coherent “backend” that all Tauri commands and the MCP/REST server can query and mutate safely, while efficiently caching device state and data.

1. Modeling the shared backend
   Create a single Backend struct
   rust
   Copy
   Edit
   pub struct Backend {
   pub device_manager: DeviceManager,
   pub device_cache: DashMap<DeviceId, CachedDevice>,   // or RwLock<HashMap<…>>
   pub last_scan: AtomicU64,                           // unix-epoch millis
   }
   Why DashMap / RwLock?
   Read-heavy, write-rare caches benefit from lock-free or multi-reader data structures; std::sync::Mutex serialises all access and can become a bottle-neck once UI, CLI, and server traffic scale.

Expose as a global singleton
rust
Copy
Edit
use once_cell::sync::Lazy;
pub static BACKEND: Lazy<Arc<Backend>> = Lazy::new(|| {
Arc::new(Backend::new())
});
Everything—including Tauri command handlers—should reference BACKEND.clone().

Register it with Tauri once for DI convenience: app.manage(BACKEND.clone());

2. Turn the watcher into an async task instead of a busy loop
   You already push USB work into std::thread::spawn, but the fixed 2 s sleep is still polling.

rusb, hidapi and Windows winusb all expose hot-plug callbacks (or, on macOS/Linux, libusb supports hotplug events). Hook those and only update on edge-triggers.

Use tokio::spawn plus a tokio::sync::watch::Sender (or tokio::sync::broadcast) to push new ApplicationState structs to both:

the UI (app_handle.emit)

any REST endpoint awaiting changes (/events, SSE, WebSocket, etc.)

3. Avoid blocking mutexes in async code
   Your REST server almost certainly runs inside the Tauri async runtime. Holding a std::sync::Mutex inside an async handler will block the entire tokio worker thread.

Replace:

rust
Copy
Edit
static DEVICE_FEATURES: Lazy<Arc<Mutex<Option<DeviceFeatures>>>> = …
with:

rust
Copy
Edit
static DEVICE_FEATURES: Lazy<Arc<tokio::sync::RwLock<Option<DeviceFeatures>>>> = …
and update calls accordingly (read().await, write().await).
Inside a purely sync context (the USB thread) you can still use blocking_write() to avoid dead-locking the executor.

4. Event emission pattern
   rust
   Copy
   Edit
   if let Err(e) = app_handle.emit_all("application:state", &payload) { … }
   emit_all guarantees delivery to any window that opens later, so you don’t need to emit a “loading” message pre-emptively in setup—the first real state from the watcher will catch all windows.

Wrap emission in a small helper to strip duplicate events (same connected & hash-eq features) to prevent spamming the UI.

5. DeviceManager instances
   You currently create three different DeviceManagers:

In spawn_usb_watcher

In server_device_manager

In unit-test or command contexts

This defeats caching. Instead:

Store one DeviceManager inside Backend.

Pass BACKEND.device_manager.clone() into both the watcher and the server.
Because it’s already Arc<…>, there’s no need for extra mutexes around it—its own interior mutability (channels, atomic flags) is enough.

6. Tauri command example
   rust
   Copy
   Edit
   #[tauri::command]
   pub async fn get_cached_features() -> Option<DeviceFeatures> {
   BACKEND.device_cache
   .get(&DeviceId::Primary)        // or however you identify it
   .map(|entry| entry.value().features.clone())
   }
   Tauri automatically injects State<'_, Arc<Backend>> if you prefer explicit DI:

rust
Copy
Edit
pub async fn get_cached_features(State(backend): State<'_, Arc<Backend>>) { … }
7. REST / MCP server integration
   Inside your server::start_server builder, put the backend into Axum’s (or Tide / Warp) extension layer:

rust
Copy
Edit
Router::new()
.route("/devices", get(list_devices))
.layer(Extension(BACKEND.clone()));
Handlers can then fetch cached devices with zero extra locking (read-only DashMap) or perform mutations (acquire a short RwLockWriteGuard).

8. Minor API & style notes
   Line	Observation	Suggested fix
   match DEVICE_FEATURES.lock()	No try_lock timeout; any panic poisons the mutex.	Use try_lock and propagate PoisonError properly—don’t silently ignore corruption.
   std::thread::sleep(…) in hot path	Blocks a core for 2 s repeatedly.	Use async sleep (tokio::time::sleep) after converting watcher to async task, or switch to event-driven hot-plug.
   Multiple app_handle.emit call sites	Risk of logging noise on every 2 s loop tick.	De-duplicate events or throttle with tokio::time::interval.
   Emitter import unused	Remove stray Emitter.
   Tag macro "{TAG} …" inside log::info!	log already lets you pass target, e.g. log::info!(target: "usb", "…").	Keep tags minimal to avoid double prefixes.

9. What a final architecture might look like
   text
   Copy
   Edit
   ┌───────────────┐      Arc<Backend>      ┌───────────────┐
   │  Tauri Window │ ─────────────────────▶ │ Device Cache  │
   └───────────────┘     (DashMap, etc.)   └───────────────┘
   ▲                                     ▲
   │ emitAll("application:state")        │
   │                                     │ watch / write
   │                         ┌───────────┴───────────┐
   │                         │   DeviceManager       │
   │                         └───────────┬───────────┘
   │                                     │
   │                         hot-plug / parse USB
   ┌───────────────┐                          events
   │ MCP / REST API│◀──────────────────────────┘
   └───────────────┘
   One authoritative Backend holds everything.

All external layers share it via Arc.

No blocking mutexes inside async code.

Hot-plug callbacks keep the cache fresh.

Next small, testable steps
Refactor DeviceManager into its own file with Send + Sync trait bounds confirmed.

Wrap it in a Backend struct and migrate all references.

Switch DEVICE_FEATURES to BACKEND.device_cache.

Add an integration test: plug/unplug a dummy device (using rusb’s mock backend) and assert REST + Tauri commands both update without duplicate allocation of DeviceManager.

Adopting this structure will give you a single, performance-friendly backend object that every layer—UI, REST, MCP, tests—can share confidently.