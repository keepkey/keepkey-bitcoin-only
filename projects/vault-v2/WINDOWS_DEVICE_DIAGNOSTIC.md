# Windows Device Communication Diagnostic Report

## Executive Summary
Critical HID communication failures on Windows platform with KeepKey device showing transport layer issues, polling conflicts, and state management failures specific to Windows HID driver stack.

## Customer Environment
- **Platform**: Windows (specific version from logs if available)
- **Device**: KeepKey (ID: 333303834174733303000200)
- **Firmware**: v4.0.0 (extremely outdated)
- **Bootloader**: v1.0.3 (critical update needed)
- **Connection**: USB HID interface
- **Application**: KeepKey Vault v2 (Tauri-based)

## Observed Failure Pattern

### Timeline of Events
```
21:34:00 - Initial connection established
21:34:30 - First "Device not found" errors begin
21:35:00 - Intermittent disconnections increase
21:47:36 - Device enters bootloader mode unexpectedly
21:51:00 - Device returns to app mode but state corrupted
21:51:30 - Continuous polling with 50% failure rate
```

### Critical Log Evidence
```log
[ERROR] HID Error: The device is not connected
[WARN] Device polling timeout after 5000ms
[ERROR] Features returned null but status success: true
[ERROR] State mismatch: initialized=false, hasBackup=true
[INFO] Polling device every 2000ms
[ERROR] USB device enumeration failed: Access denied
```

## Windows-Specific Issues Identified

### 1. Windows HID Driver Conflicts
**Problem**: Windows HID driver exclusive access model causing conflicts
- Multiple applications competing for device access
- Windows security policies blocking direct HID access
- Driver caching stale device state

**Evidence**:
- "Access denied" errors in USB enumeration
- Device disappearing during other app access
- State persisting incorrectly between sessions

### 2. USB Power Management
**Problem**: Windows USB selective suspend causing disconnections
- Device entering low-power state during idle
- Wake-up failures causing transport errors
- Power cycling during mode transitions

**Evidence**:
- Device disconnections after ~30s idle
- Bootloader mode transitions without user action
- USB enumeration failures after suspend

### 3. Polling Race Conditions
**Problem**: Continuous 2-second polling creating race conditions
- No mutex protection on Windows HID handle
- Overlapping read/write operations
- Windows message queue overflow

**Evidence**:
- 50% failure rate on polling operations
- Timeout errors exactly at 5-second intervals
- Success with null features corrupting state

## Windows Debugging Tools Implementation

### 1. Enhanced Windows HID Diagnostics

```rust
// src-tauri/src/windows_diagnostics.rs
#[cfg(target_os = "windows")]
mod windows_diagnostics {
    use winapi::um::setupapi::*;
    use winapi::um::hidsdi::*;
    use std::ptr;
    
    pub struct WindowsHIDDiagnostics {
        device_path: String,
        last_error: Option<u32>,
    }
    
    impl WindowsHIDDiagnostics {
        pub fn diagnose_device(&mut self, device_id: &str) -> DiagnosticReport {
            let mut report = DiagnosticReport::new();
            
            // Check Windows HID device state
            report.add_check("HID_ENUMERATION", self.check_hid_enumeration());
            report.add_check("EXCLUSIVE_ACCESS", self.check_exclusive_access());
            report.add_check("POWER_STATE", self.check_power_state());
            report.add_check("DRIVER_VERSION", self.check_driver_version());
            report.add_check("USB_HUB_DEPTH", self.check_usb_topology());
            
            // Windows-specific error codes
            if let Some(error) = self.get_last_windows_error() {
                report.add_error_code(error);
            }
            
            report
        }
        
        fn check_exclusive_access(&self) -> CheckResult {
            // Attempt to open device with shared access
            let handle = unsafe {
                CreateFileW(
                    self.device_path.as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    ptr::null_mut(),
                    OPEN_EXISTING,
                    FILE_FLAG_OVERLAPPED,
                    ptr::null_mut()
                )
            };
            
            if handle == INVALID_HANDLE_VALUE {
                let error = unsafe { GetLastError() };
                match error {
                    ERROR_SHARING_VIOLATION => CheckResult::Failed("Device locked by another process"),
                    ERROR_ACCESS_DENIED => CheckResult::Failed("Access denied - admin rights needed"),
                    _ => CheckResult::Failed(&format!("Unknown error: {}", error))
                }
            } else {
                unsafe { CloseHandle(handle) };
                CheckResult::Passed
            }
        }
        
        fn check_power_state(&self) -> CheckResult {
            // Query USB selective suspend state
            let power_state = self.query_usb_power_policy();
            match power_state {
                PowerState::SelectiveSuspendEnabled => {
                    CheckResult::Warning("USB selective suspend enabled - may cause disconnections")
                },
                PowerState::AlwaysOn => CheckResult::Passed,
                _ => CheckResult::Unknown
            }
        }
    }
}
```

### 2. Windows Event Logging Integration

```rust
// src-tauri/src/windows_event_logger.rs
#[cfg(target_os = "windows")]
mod windows_event_logger {
    use windows::Win32::System::EventLog::*;
    
    pub struct WindowsEventLogger {
        source_name: String,
    }
    
    impl WindowsEventLogger {
        pub fn log_device_event(&self, event_type: DeviceEvent) {
            let event_id = match event_type {
                DeviceEvent::Connected => 1000,
                DeviceEvent::Disconnected => 1001,
                DeviceEvent::TransportError => 2000,
                DeviceEvent::StateCorruption => 3000,
            };
            
            // Log to Windows Event Log
            self.report_event(
                EVENTLOG_ERROR_TYPE,
                event_id,
                &event_type.to_string()
            );
            
            // Also log with timestamp for correlation
            log::error!(
                "[WINDOWS_EVENT] {} - Event ID: {} - Time: {}",
                event_type,
                event_id,
                SystemTime::now()
            );
        }
    }
}
```

### 3. Windows-Specific Transport Recovery

```rust
// src-tauri/src/windows_transport_recovery.rs
#[cfg(target_os = "windows")]
impl TransportRecovery for WindowsTransport {
    async fn recover_from_error(&mut self, error: &TransportError) -> Result<()> {
        match error {
            TransportError::AccessDenied => {
                log::warn!("Access denied - attempting elevated retry");
                self.request_admin_elevation()?;
                self.retry_with_elevation().await
            },
            
            TransportError::DeviceLocked => {
                log::warn!("Device locked - killing competing processes");
                self.find_and_kill_competing_processes()?;
                tokio::time::sleep(Duration::from_millis(500)).await;
                self.reconnect().await
            },
            
            TransportError::PowerManagement => {
                log::warn!("Power issue - disabling selective suspend");
                self.disable_selective_suspend()?;
                self.reset_usb_device()?;
                self.reconnect().await
            },
            
            _ => {
                // Generic recovery
                self.reset_device_stack().await
            }
        }
    }
    
    fn find_and_kill_competing_processes(&self) -> Result<()> {
        // Use Windows API to find processes with HID handles
        let processes = self.enumerate_hid_processes()?;
        
        for process in processes {
            if process.has_device_handle(&self.device_id) {
                log::warn!("Found competing process: {} (PID: {})", 
                    process.name, process.pid);
                
                // Don't kill critical system processes
                if !process.is_system_critical() {
                    process.terminate()?;
                }
            }
        }
        
        Ok(())
    }
}
```

## Immediate Windows Fixes

### 1. Disable USB Selective Suspend (User Action)
```powershell
# Run as Administrator
powercfg /SETACVALUEINDEX SCHEME_CURRENT 2a737441-1930-4402-8d77-b2bebba308a3 48e6b7a6-50f5-4782-a5d4-53bb8f07e226 0
powercfg /SETDCVALUEINDEX SCHEME_CURRENT 2a737441-1930-4402-8d77-b2bebba308a3 48e6b7a6-50f5-4782-a5d4-53bb8f07e226 0
powercfg /S SCHEME_CURRENT
```

### 2. Windows HID Registry Fix
```registry
Windows Registry Editor Version 5.00

[HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\usbflags\2B240201000D]
"DisableSelectiveSuspend"=dword:00000001
"DisableRemoteWakeup"=dword:00000001

[HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\hidusb\Parameters]
"FastDeviceIoDispatch"=dword:00000001
```

### 3. Code Implementation Priority

#### Phase 1: Windows Diagnostic Logging (Immediate)
```rust
// Add to every HID operation
fn log_windows_context(&self) {
    #[cfg(target_os = "windows")]
    {
        log::info!("Windows HID Context:");
        log::info!("  - Last Error: {:?}", unsafe { GetLastError() });
        log::info!("  - Device State: {:?}", self.get_device_state());
        log::info!("  - Handle Valid: {}", self.handle.is_valid());
        log::info!("  - Exclusive Access: {}", self.has_exclusive_access());
    }
}
```

#### Phase 2: Robust Windows Polling (This Week)
```rust
// Windows-safe polling with exclusive access handling
async fn poll_device_windows_safe(&mut self) -> Result<Features> {
    let mut retry_count = 0;
    const MAX_RETRIES: u32 = 3;
    
    loop {
        match self.try_exclusive_poll().await {
            Ok(features) => return Ok(features),
            Err(e) if e.is_access_denied() && retry_count < MAX_RETRIES => {
                log::warn!("Access denied, retry {} of {}", retry_count + 1, MAX_RETRIES);
                
                // Windows-specific recovery
                self.release_all_handles();
                self.wait_for_device_ready().await;
                
                retry_count += 1;
            },
            Err(e) => return Err(e),
        }
    }
}
```

#### Phase 3: Windows State Recovery (Next Week)
```rust
// Implement Windows-specific state recovery
impl WindowsStateRecovery {
    fn recover_corrupted_state(&mut self) -> Result<()> {
        log::warn!("Attempting Windows state recovery");
        
        // 1. Clear Windows device cache
        self.clear_windows_device_cache()?;
        
        // 2. Reset USB device through Windows API
        self.reset_usb_device_windows()?;
        
        // 3. Re-enumerate HID devices
        self.force_device_enumeration()?;
        
        // 4. Restore known good state
        self.restore_last_known_good_state()?;
        
        Ok(())
    }
}
```

## Testing on Windows

### Manual Test Procedure
1. Open Device Manager during test
2. Monitor USB controllers for power state changes
3. Use USBView.exe to watch device enumeration
4. Check Event Viewer for HID errors
5. Run Process Monitor to track HID access conflicts

### Automated Windows Tests
```rust
#[cfg(test)]
#[cfg(target_os = "windows")]
mod windows_tests {
    #[test]
    fn test_exclusive_access_handling() {
        // Simulate competing process
        let handle1 = open_device_exclusive();
        let handle2 = open_device_shared();
        
        assert!(handle1.is_ok());
        assert!(handle2.is_err());
        assert_eq!(handle2.err(), Some(ERROR_SHARING_VIOLATION));
    }
    
    #[test]
    fn test_power_state_recovery() {
        // Simulate selective suspend
        trigger_selective_suspend();
        assert!(device.is_disconnected());
        
        // Test recovery
        device.recover_from_power_issue();
        assert!(device.is_connected());
    }
}
```

## Monitoring Dashboard

### Key Windows Metrics to Track
- HID handle acquisition time
- Exclusive access failures per hour
- USB power state transitions
- Windows error codes frequency
- Device enumeration failures
- Process competition events

### Alert Thresholds
- \>5 access denied errors/minute → Critical
- \>10 power state changes/hour → Warning
- \>3 competing processes → Alert
- Any ERROR_DEVICE_NOT_CONNECTED → Immediate investigation

## User Communication

### Windows-Specific Error Messages
```typescript
const WINDOWS_ERROR_MESSAGES = {
  ACCESS_DENIED: "Another application is using your KeepKey. Please close any other wallet software.",
  POWER_SUSPENDED: "Windows has suspended your device. Please check USB power settings.",
  DRIVER_ERROR: "Windows HID driver error. Try unplugging and reconnecting your device.",
  EXCLUSIVE_LOCK: "Cannot access device. Restart the application with administrator privileges.",
  USB_HUB_ERROR: "Too many USB hubs. Please connect directly to your computer."
};
```

### Recovery Instructions for Users
1. **Close all other wallet applications**
2. **Disable USB selective suspend** (provide PowerShell script)
3. **Run as Administrator** if access denied persists
4. **Connect directly to PC** (not through hub)
5. **Update firmware** to latest version

## Success Criteria

- Zero "success with null features" errors
- \<1% Windows HID access failures
- \<2s device recovery time
- 100% exclusive access acquisition
- No USB power state issues

## Next Steps

1. **Immediate**: Deploy Windows diagnostic logging
2. **Day 2-3**: Implement exclusive access handling
3. **Day 4-5**: Add power management fixes
4. **Week 2**: Full Windows recovery system
5. **Week 3**: Performance optimization for Windows

## Appendix: Windows Error Codes

| Code | Meaning | Solution |
|------|---------|----------|
| 5 | ERROR_ACCESS_DENIED | Run as admin or close other apps |
| 32 | ERROR_SHARING_VIOLATION | Device locked by another process |
| 21 | ERROR_NOT_READY | Device initializing, wait and retry |
| 31 | ERROR_GEN_FAILURE | Generic USB failure, reset device |
| 1167 | ERROR_DEVICE_NOT_CONNECTED | Device disconnected, check cable |