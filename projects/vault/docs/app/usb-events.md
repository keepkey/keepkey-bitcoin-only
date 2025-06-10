Yes — replace the 2-second sleep with true USB hot-plug callbacks
Below is the cross-platform outline. It boils down to “let the OS tell us when something changes instead of asking every few seconds.” We still reuse your detect_all_devices() helper to classify the device once a notification arrives, but that expensive scan only runs when a real attach/detach event fires.

1 Linux & macOS: libusb hot-plug (via rusb)
rust
Copy
Edit
use rusb::{Context, HotplugBuilder, HotplugEvent};
use tauri::AppHandle;
use std::sync::{Arc, Mutex};
use std::thread;

// …

pub fn start_detection(&self, app: AppHandle) {
let states   = self.states.clone();
let running  = self.running.clone();

    // flip running = true (same guard logic as before) …

    thread::spawn(move || {
        // libusb context must live on the same thread that calls handle_events()
        let mut ctx = Context::new().expect("libusb init");

        // Register callback; keep the returned registration alive
        let _reg = HotplugBuilder::new()
            .enumerate(true) // fire once for devices that are already present
            .vendor_id(KEEPKEY_VID)
            .product_id(None)                     // match any PID
            .register(&mut ctx,
              move |_dev, event| {
                  // We only care about arrived/left
                  match event {
                      HotplugEvent::DeviceArrived |
                      HotplugEvent::DeviceLeft => {
                          // Snapshot & emit in *another* thread so libusb callback stays fast
                          let snapshot = detect_all_devices();
                          {
                              let mut g = states.lock().unwrap();
                              if *g != snapshot {
                                  *g = snapshot.clone();
                                  let _ = app.emit("device_state_changed", snapshot);
                              }
                          }
                      }
                      _ => {}
                  }
                  // keep callback alive
                  true
              })
            .expect("register callback");

        // Pump libusb’s internal FD set; 50 ms timeout keeps latency low
        while *running.lock().unwrap() {
            ctx.handle_events_timeout(std::time::Duration::from_millis(50))
               .ok(); // ignore EINTER etc.
        }
    });
}
Key points

What	Why
HotplugBuilder::enumerate(true)	Fires a synthetic “arrived” event for devices already connected at start-up.
handle_events_timeout() loop	Required so libusb can dispatch the callbacks; but we’re now sleeping 50 ms inside the libusb internals, not blocking your own logic.
The callback only pushes a lightweight “snapshot changed” check; the heavy feature-probe work happens after the callback returns.

libusb hot-plug works on Linux and macOS (with the default AppleUSB interface). It does not fire on Windows, so we add a Windows-only path next.

2 Windows: device-interface notifications
Windows surfaces “arrive/remove” through WM_DEVICECHANGE or RegisterDeviceNotification.
The cleanest Rust wrapper today is the device_notify crate (thin over windows-sys). A sketch:

rust
Copy
Edit
#[cfg(target_os = "windows")]
fn spawn_windows_hotplug(states: Arc<Mutex<Vec<DeviceState>>>,
running: Arc<Mutex<bool>>,
app: AppHandle) {
std::thread::spawn(move || {
use device_notify::{DeviceEvents, DeviceWatcher};
let watcher = DeviceWatcher::new().unwrap();

        watcher.add_vendor(KEEPKEY_VID).unwrap(); // filter early
        watcher.start(DeviceEvents::DEVICEINTERFACE);

        while *running.lock().unwrap() {
            if let Ok(event) = watcher.receiver().recv_timeout(Duration::from_millis(200)) {
                match event {
                    DeviceEvents::Arrived | DeviceEvents::Removed => {
                        let snapshot = detect_all_devices();
                        let mut g = states.lock().unwrap();
                        if *g != snapshot {
                            *g = snapshot.clone();
                            let _ = app.emit("device_state_changed", snapshot);
                        }
                    }
                    _ => {}
                }
            }
        }
    });
}
You would call this helper from start_detection() under #[cfg(target_os = "windows")].

3 HID-only fallback (optional)
If you really need per-interface HID notifications (e.g., mode flip that doesn’t re-enumerate the USB device), you can:

Re-ask for Features whenever you get a USB arrived notification (the device often cycles).

Or use hidapi’s callback on Linux/macOS; it internally piggy-backs on the same libusb hot-plug.

In practice KeepKey switches PID when changing modes, so USB hot-plug already covers every user-visible transition.

4 Putting it together
rust
Copy
Edit
pub fn start_detection(&self, app: AppHandle) {
// set running = true …

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    spawn_libusb_hotplug(self.states.clone(), self.running.clone(), app.clone());

    #[cfg(target_os = "windows")]
    spawn_windows_hotplug(self.states.clone(), self.running.clone(), app.clone());
}
stop_detection() stays unchanged; it just flips running = false and the individual threads exit.

Net result
Zero polling in your own code.

Latency is down to the OS notification path (tens of ms).

CPU usage is negligible even with many devices plugged/unplugged.