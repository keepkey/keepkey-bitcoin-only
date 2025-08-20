## KeepKey Bitcoin-Only — USB Connection Lifecycle Report

### Summary
- **Symptom**: Device appears to connect/disconnect for every request (e.g., get xpub, get address).
- **Root causes**:
    - The per-device worker explicitly drops its transport after handling each command.
    - Fallback paths create new transports per call instead of reusing a persistent one.
    - USB transport performs a hard `reset()` immediately after `open()`, which can trigger OS re-enumeration (notably on macOS).
- **What v8 does**: Keeps a single transport per device alive in a queue worker and avoids hard reset-on-open.
- **Impact**: Extra latency, increased flakiness on macOS, and visible connect/reconnect behavior for every operation.

---

### Evidence in keepkey-bitcoin-only

1) Transport is dropped after each command

```278:286:projects/keepkey-rust/device_queue.rs
        self.metrics.record_operation(queue_wait, device_rtt, total_time);

    // Always drop transport after each command to avoid exclusive handle issues,
    // it will be recreated lazily on the next command.
    if self.transport.is_some() {
        info!("🔌 Releasing transport handle for device {} after operation", self.device_id);
    }
    self.transport = None;
    
    Ok(())
```

2) Fallback path creates a fresh transport per message

```1530:1596:projects/keepkey-rust/commands.rs
/// Send message to device (creates transport on-demand)
async fn send_message_to_device(device_id: &str, message: crate::messages::Message) -> Result<crate::messages::Message, String> {
    // ...
    if in_pin_flow || in_recovery_flow {
        // ...
        // Create transport
        let mut transport = create_device_transport(&target_device.device).await?;
        // ...
    } else {
        // Try queue first; on failure, fall back to direct transport
        match send_message_via_queue(Some(device_id), message.clone()).await {
            Ok(response) => Ok(response),
            Err(_) => {
                // Fallback: create transport again
                let mut transport = create_device_transport(&target_device.device).await?;
                let mut handler = transport.with_standard_handler();
                handler.handle(message)
            }
        }
    }
}
```

3) New queue worker spawned on demand (not reused via registry)

```2471:2489:projects/keepkey-rust/commands.rs
async fn get_device_queue_or_fallback(device_id: &str) -> Result<DeviceQueueHandle, String> {
    // ... find device entry ...
    // Create a new queue handle using the device queue factory
    let queue_handle = crate::device_queue::DeviceQueueFactory::spawn_worker(
        device_id.to_string(), 
        device_entry.device.clone()
    );
    Ok(queue_handle)
}
```

4) USB transport issues a hard reset immediately after open

```31:37:projects/keepkey-rust/transport/usb.rs
        let locked_handle = Arc::get_mut(&mut handle_arc)
            .ok_or(rusb::Error::Other)?
            .get_mut()
            .map_err(|_| rusb::Error::Other)?;
        
        locked_handle.reset()?;
```

---

### Contrast with v8 (keepkey-vault)

1) Transport is kept alive across commands in the device worker

```316:320:../../keepkey-vault/projects/keepkey-usb/src/device/device_queue.rs
        self.metrics.record_operation(queue_wait, device_rtt, total_time);

        // Keep transport alive across commands for performance.
        // It will be recreated on demand by ensure_transport() only after errors
        // or explicitly during disruptive operations (e.g., firmware/bootloader updates)
        Ok(())
```

2) No hard reset after open (reset is explicitly avoided/commented)

```31:37:../../keepkey-vault/projects/keepkey-usb/src/transport/usb.rs
        let locked_handle = Arc::get_mut(&mut handle_arc)
            .ok_or(rusb::Error::Other)?
            .get_mut()
            .map_err(|_| rusb::Error::Other)?;
        
        // locked_handle.reset()?;
```

3) Requests are funneled through a single per-device queue worker (persistent transport, caching, metrics), and server code (`keepkey-server`) reuses that queue rather than creating fresh transports.

---

### Impact
- **Performance**: Repeated open/close increases per-request latency and USB enumeration overhead.
- **Stability (macOS)**: `reset()` after open can race with OS re-enumeration leading to transient "device not found"/reconnect behavior.
- **UX**: Users see connect/reconnect events during routine operations (xpub/address fetches).

---

### Minimal, targeted changes to fix bitcoin-only

1) Keep the transport alive in the queue worker
- In `projects/keepkey-rust/device_queue.rs`, remove the unconditional transport drop after each command and follow the v8 pattern: only clear the transport on errors or during disruptive ops (firmware/bootloader updates).

2) Avoid hard reset on open
- In `projects/keepkey-rust/transport/usb.rs`, remove `locked_handle.reset()?;` to match v8 behavior and prevent macOS re-enumeration.

3) Reuse a single queue per device
- Update `get_device_queue_or_fallback(...)` to first attempt `device_registry::get_device_queue_handle(device_id)` and return it when present.
- If absent, spawn once via `DeviceQueueFactory::spawn_worker(...)` and immediately store it with `device_registry::add_or_update_device_with_queue(...)` so subsequent requests reuse the same handle.

4) Limit direct transport creation to special sessions
- Keep direct `create_device_transport(...)` only for explicit recovery/PIN sessions, and consider caching a per-session transport handle rather than recreating it per message.

---

### Expected result after changes
- Single open/persistent transport per device; no per-request reconnect churn.
- Reduced latency and fewer enumeration races on macOS.
- Behavior aligned with v8’s device queue and transport lifecycle.

---

### Nice-to-have (optional)
- Route more operations through the queue (including PIN flows where feasible) to centralize transport ownership.
- Move blocking USB/HID I/O into `spawn_blocking` where applicable to avoid runtime starvation.
- Add simple metrics for queue depth and per-request RTT to spot regressions.


