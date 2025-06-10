# TroubleshootingWizard Planning Document

## Problem Statement

When KeepKey devices are detected via USB but fail to establish communication (features fetch fails), users currently see broken buttons in the settings dialog and no clear guidance on how to resolve the issue.

**Current Issue:**
- Device detected: `keepkey_2b24_0002_bus0_addr2`
- Features fail to load: `firmwareVersion: "Unknown"`, `bootloaderMode: true`
- All buttons (Update Bootloader, Update Firmware, Create Wallet) are non-functional
- User has no guidance on troubleshooting steps

## Solution: TroubleshootingWizard

### New Blocking Action Type
**Priority: 110** (highest priority, even above bootloader updates)

```rust
#[serde(rename = "device_communication_failure")]
DeviceCommunicationFailure,
```

### Auto-Launch Criteria
The TroubleshootingWizard should auto-launch when:
1. Device is detected as KeepKey (VID: 2b24, PID: 0001 or 0002)
2. Device features fetch fails after max retries (currently 3 attempts)
3. Device shows `firmwareVersion: "Unknown"` or `features: null`
4. Device communication timeouts occur repeatedly

### UI Changes in KeepKeyDeviceList

**Current State (Broken):**
```tsx
// Show all buttons even when device can't communicate
<Button>Update Bootloader</Button>
<Button>Update Firmware</Button> 
<Button>Create Wallet</Button>
```

**New State (Communication Failed):**
```tsx
// Show warning and troubleshooter only
<Box bg="yellow.100" borderColor="yellow.400" borderWidth={2}>
  <HStack>
    <Icon as={FaExclamationTriangle} color="yellow.600" />
    <Text color="yellow.800">Device detected but not communicating</Text>
  </HStack>
</Box>
<Button colorScheme="yellow">Launch Troubleshooter</Button>
```

### TroubleshootingWizard Steps

#### Step 0: Problem Detection
- **Title:** "Device Communication Issue Detected"
- **Content:** 
  - Explain that KeepKey was detected but can't communicate
  - Show device info (VID/PID, connection status)
  - "Don't worry - this is usually fixable!"

#### Step 1: Basic Troubleshooting
- **Title:** "Let's Try Basic Solutions"
- **Content:**
  - **Cable Check:** "Try a different USB cable (data cable, not charge-only)"
  - **Port Check:** "Try a different USB port (preferably USB 2.0)" 
  - **Direct Connection:** "Remove any USB hubs or extensions"
  - **Restart:** "Unplug device for 10 seconds, then reconnect"
- **Action:** Auto-retry connection after each step

#### Step 2: Advanced Troubleshooting  
- **Title:** "Advanced Recovery Steps"
- **Content:**
  - **Bootloader Mode:** "Hold button while connecting USB cable"
  - **Device Reset:** "Hold both buttons for 12 seconds to reset"
  - **Driver Issues:** Platform-specific driver troubleshooting
    - **Windows:** Check Device Manager, reinstall drivers
    - **macOS:** Check System Preferences > Security & Privacy
    - **Linux:** Check udev rules, permissions

#### Step 3: Force Recovery Mode
- **Title:** "Emergency Recovery"
- **Content:**
  - **Factory Reset:** Option to force device into recovery mode
  - **Contact Support:** If all else fails, provide support contact info
  - **Device Information:** Export device logs/info for support

#### Step 4: Success/Resolution
- **Title:** "Communication Restored!" or "Contact Support"
- **Content:** 
  - If successful: "Your KeepKey is now communicating properly"
  - If failed: "Please contact support with the diagnostic information"

### Backend Integration

#### New Blocking Action Creation
```rust
impl BlockingAction {
    pub fn new_communication_failure(device_id: &str, error_details: &str) -> Self {
        Self {
            device_id: device_id.to_string(),
            action_type: BlockingActionType::DeviceCommunicationFailure,
            message: format!("Device {} cannot communicate: {}", device_id, error_details),
            priority: 110, // Highest priority - above bootloader updates
            current_version: None,
            required_version: None,
        }
    }
}
```

#### Trigger Logic in DeviceController
```rust
// In device_controller.rs after max retries reached
if retry_count >= MAX_RETRIES {
    log::warn!("Max retries reached for device {}, creating communication failure action", device_id);
    
    let action = BlockingAction::new_communication_failure(
        &device_id,
        "Failed to fetch device features after multiple attempts"
    );
    
    blocking_actions.lock().unwrap().add_action(action);
    // Emit blocking actions update to trigger frontend
}
```

### Frontend Components Structure

```
src/components/TroubleshootingWizard/
├── TroubleshootingWizard.tsx           // Main wizard component  
├── steps/
│   ├── Step0ProblemDetection.tsx       // Explain the issue
│   ├── Step1BasicTroubleshooting.tsx   // Cable, port, restart
│   ├── Step2AdvancedRecovery.tsx       // Bootloader, reset, drivers
│   ├── Step3ForceRecovery.tsx          // Emergency options
│   └── Step4Resolution.tsx             // Success or support contact
└── components/
    ├── ConnectionTester.tsx            // Test device connection
    ├── DriverChecker.tsx              // Platform-specific driver checks
    └── DiagnosticExporter.tsx         // Export device info for support
```

### Dialog Context Integration

```tsx
// Add to DialogContext.tsx
export function useTroubleshootingWizard() {
  const { show, hide, isShowing } = useDialog();
  return {
    show: (props: {
      deviceId: string;
      errorDetails: string;
      onResolved?: () => void;
      onContactSupport?: (diagnostics: any) => void;
    }) => {
      const dialogId = `troubleshooting-wizard-${props.deviceId}`;
      show({
        id: dialogId,
        component: React.lazy(() => import('../components/TroubleshootingWizard/TroubleshootingWizard')),
        props: { ...props },
        priority: 'critical', // Highest priority
        persistent: true,
      });
    },
    hide: (deviceId: string) => hide(`troubleshooting-wizard-${deviceId}`),
    isShowing: (deviceId: string) => isShowing(`troubleshooting-wizard-${deviceId}`),
  };
}
```

### BlockingActionsContext Integration

```tsx
// Add to BlockingActionsContext.tsx switch statement
case BlockingActionType.DeviceCommunicationFailure:
  const isWizardShown = troubleshootingWizard.isShowing(highestPriorityAction.device_id);
  
  if (!isWizardShown) {
    troubleshootingWizard.show({
      deviceId: highestPriorityAction.device_id,
      errorDetails: highestPriorityAction.message,
      onResolved: async () => {
        await resolveAction(highestPriorityAction.device_id, BlockingActionType.DeviceCommunicationFailure);
        fetchActions();
      },
      onContactSupport: (diagnostics) => {
        // Handle support contact with diagnostic info
      }
    });
  }
  break;
```

## Implementation Priority

1. **Backend Changes** (High Priority)
   - Add `DeviceCommunicationFailure` to `BlockingActionType` enum
   - Add trigger logic in `device_controller.rs` 
   - Add new blocking action creation function

2. **Frontend Wizard** (High Priority)
   - Create `TroubleshootingWizard` component with 5 steps
   - Add to `DialogContext` integration
   - Add to `BlockingActionsContext` handling

3. **UI Updates** (Medium Priority)
   - Update `KeepKeyDeviceList.tsx` to hide broken buttons and show warning
   - Add "Launch Troubleshooter" button for communication-failed devices

4. **Platform-Specific Features** (Lower Priority)
   - Driver checking for Windows/macOS/Linux
   - Advanced diagnostic export functionality
   - Integration with support systems

## Success Metrics

- **User Experience:** Users with communication issues get immediate, clear guidance
- **Issue Resolution:** Higher success rate for device communication problems
- **Support Reduction:** Fewer support tickets for basic communication issues
- **Device Recovery:** Better recovery rate for devices in problematic states

## Existing Components to Leverage

- **Wizard Pattern:** Copy structure from `FirmwareUpdateWizard.tsx`
- **Dialog System:** Use existing `DialogContext.tsx` priority system
- **Blocking Actions:** Extend existing `BlockingActionsContext.tsx`
- **Device Management:** Integrate with existing `device_controller.rs` retry logic 