# KeepKey Device Update Architecture

This document outlines the architecture for handling device updates (bootloader and firmware) in the KeepKey Desktop application.

## Overview

The KeepKey Desktop application needs to handle two types of device updates:

1. **Bootloader Updates**: Critical updates that must be applied before any other action
2. **Firmware Updates**: Important updates that can be applied after bootloader updates

These updates are managed through a priority-based blocking actions system that ensures updates are applied in the correct order.

## Key Components

### 1. Blocking Actions System

The blocking actions system tracks actions that must be completed before the device can be used normally.

```
BlockingAction
├── id: UUID
├── device_id: String
├── action_type: ActionType
│   ├── BootloaderUpdate(current_version, required_version)
│   ├── FirmwareUpdate(current_version, target_version)
│   └── DeviceInitialization
├── priority: Priority (HIGH=100, MEDIUM=50, LOW=10)
├── created_at: DateTime
├── status: Status (Pending, InProgress, Completed, Failed)
```

### 2. Priority Queue

Actions are processed in priority order, with bootloader updates taking precedence over firmware updates:

- **Bootloader Update**: Priority 100 (Highest)
- **Device Initialization**: Priority 80 (Medium)
- **Firmware Update**: Priority 50 (Lower)

### 3. Backend Components

- **BlockingActionsRegistry**: Stores and manages all blocking actions
- **BlockingActionsState**: Provides thread-safe access to the registry
- **ApplicationState**: Tracks and reports the count of blocking actions

### 4. Frontend Components

- **BlockingActionsContext**: React context that manages blocking actions state
- **BlockingActionsProvider**: Provider component that listens for backend events
- **BootloaderUpdateWizard**: Multi-step wizard for bootloader updates
- **FirmwareUpdateWizard**: Multi-step wizard for firmware updates

## Workflow

1. **Detection**: When a device connects, the backend checks for required updates:
   - Bootloader version check (`utils::is_version_older()`)
   - Firmware version check (`utils::is_version_older()`)

2. **Action Creation**: For each required update, a blocking action is created and added to the registry with the appropriate priority

3. **Event Emission**: The backend emits events to notify the frontend of new blocking actions

4. **UI Response**: 
   - The BlockingActionsContext processes the events and updates its state
   - UI components display the count of pending actions
   - The appropriate update button is enabled based on available actions

5. **Wizard Launch**:
   - When an update is required, the appropriate wizard is automatically launched
   - Only the highest priority wizard is shown at one time
   - After completion, the next highest priority action is processed

## Implementation Guidelines

### Backend 

1. **Detecting Required Updates**:
```rust
// Check bootloader version
if let Some(bootloader_version) = features.bootloader_version.as_deref() {
    let required_version = "2.1.4"; // Required bootloader version
    if utils::is_version_older(bootloader_version, required_version) {
        // Create bootloader update blocking action with HIGHEST priority
        registry.add_action(BlockingAction::new_bootloader_update(
            device_id, bootloader_version, required_version, Priority::HIGH));
    }
}

// Check firmware version
if let Some(firmware_version) = features.firmware_version.as_deref() {
    let latest_version = "7.7.0"; // Latest firmware version
    if utils::is_version_older(firmware_version, latest_version) {
        // Create firmware update blocking action with LOWER priority
        registry.add_action(BlockingAction::new_firmware_update(
            device_id, firmware_version, latest_version, Priority::MEDIUM));
    }
}
```

2. **Managing Blocking Actions**:
```rust
impl BlockingActionsRegistry {
    // Get the highest priority pending action for a specific device
    pub fn get_highest_priority_action_for_device(&self, device_id: &str) -> Option<&BlockingAction> {
        self.actions.values()
            .filter(|action| action.device_id == device_id && action.status == Status::Pending)
            .max_by_key(|action| action.priority)
    }
    
    // Get all pending actions for a device
    pub fn get_pending_actions_for_device(&self, device_id: &str) -> Vec<&BlockingAction> {
        self.actions.values()
            .filter(|action| action.device_id == device_id && action.status == Status::Pending)
            .collect()
    }
}
```

### Frontend

1. **Automatic Wizard Launch**:
```typescript
function useBlockingActionsEffect() {
  const { actions, resolveAction } = useBlockingActions();
  const { openDialog } = useDialog();
  
  // Launch appropriate wizard based on highest priority action
  useEffect(() => {
    const highestPriorityAction = getHighestPriorityAction(actions);
    
    if (highestPriorityAction) {
      switch (highestPriorityAction.action_type) {
        case 'bootloader_update':
          openDialog({
            component: BootloaderUpdateWizard,
            props: {
              deviceId: highestPriorityAction.device_id,
              currentVersion: highestPriorityAction.current_version,
              requiredVersion: highestPriorityAction.required_version,
              onComplete: () => resolveAction(highestPriorityAction.id)
            },
            persistent: true
          });
          break;
        
        case 'firmware_update':
          // Only show if there's no pending bootloader update
          if (!hasPendingBootloaderUpdate(actions, highestPriorityAction.device_id)) {
            openDialog({
              component: FirmwareUpdateWizard,
              props: {
                deviceId: highestPriorityAction.device_id,
                currentVersion: highestPriorityAction.current_version,
                targetVersion: highestPriorityAction.target_version,
                onComplete: () => resolveAction(highestPriorityAction.id)
              },
              persistent: true
            });
          }
          break;
      }
    }
  }, [actions, openDialog, resolveAction]);
}
```

2. **UI Components**:
```typescript
function DeviceActions({ deviceId }) {
  const { actions } = useBlockingActions();
  const deviceActions = actions.filter(action => action.device_id === deviceId);
  
  const bootloaderAction = deviceActions.find(a => a.action_type === 'bootloader_update');
  const firmwareAction = deviceActions.find(a => a.action_type === 'firmware_update');
  
  return (
    <div className="device-actions">
      <Button 
        disabled={!bootloaderAction} 
        onClick={() => handleBootloaderUpdate(deviceId, bootloaderAction)}>
        Update Bootloader
      </Button>
      
      <Button 
        disabled={!firmwareAction || bootloaderAction} 
        onClick={() => handleFirmwareUpdate(deviceId, firmwareAction)}>
        Update Firmware
      </Button>
    </div>
  );
}
```

## Important Constraints

1. **Bootloader Updates First**: Always prioritize bootloader updates over firmware updates
2. **One Wizard at a Time**: Only show one update wizard at a time
3. **Windows HID Handling**: Properly handle device reconnection after updates, especially on Windows
4. **Persistent Wizards**: Update wizards cannot be dismissed until complete or explicitly canceled
5. **Visual Indicators**: Clearly indicate which updates are required in the UI

## Testing Considerations

1. Test with multiple connected devices requiring different updates
2. Test the priority ordering to ensure bootloader updates are always processed first
3. Test device reconnection after updates, especially on Windows
4. Test error handling and retry flows
5. Test the UI to ensure it properly reflects the available actions
