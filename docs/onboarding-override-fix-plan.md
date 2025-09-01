# Onboarding Override Fix - Technical Planning Document

## Problem Statement

The onboarding flow is currently being interrupted and bypassed by device connection activities, including:
- PIN dialogs appearing before users learn about their KeepKey
- Passphrase dialogs breaking the educational flow
- Vault interface launching automatically when device connects
- Setup wizards and device update managers triggering independently

**Core Issue**: Device connection events and interactions are initiated before the user completes the educational onboarding sequence.

## Current Architecture Analysis

### Onboarding Flow Components

1. **OnboardingWizard** (`/src/components/OnboardingWizard/OnboardingWizard.tsx`)
   - 5-step educational flow: Language → App Settings → Security → Recovery → Complete
   - Sets `onboarding_completed` flag in database when finished
   - Triggered by `shouldShowOnboarding` logic

2. **Onboarding State Management** (`/src/hooks/useOnboardingState.ts`)
   - Checks `is_first_time_install()` and `is_onboarded()` from backend
   - `shouldShowOnboarding = isFirstTime === true || isOnboarded === false`
   - Cached to prevent duplicate backend calls

### Device Connection Entry Points

**Primary Entry Points Bypassing Onboarding:**

1. **App.tsx Event Listeners** (Lines 282-498)
   - `frontend_ready` signal triggers device scanning immediately
   - `device:ready` event launches VaultInterface
   - `passphrase_request` event shows passphrase dialogs
   - No onboarding state checks before device interactions

2. **useDeviceInteraction Hook** (`/src/hooks/useDeviceInteraction.ts`)
   - Listens for PIN/passphrase/button requests globally
   - Shows dialogs immediately when device events occur
   - No awareness of onboarding state

3. **DeviceUpdateManager** (`/src/components/DeviceUpdateManager.tsx`)
   - Handles bootloader updates, firmware updates, wallet creation
   - Triggers setup wizards independently of onboarding
   - Shows PIN unlock dialogs when device requires authentication

4. **WalletContext** (`/src/contexts/WalletContext.tsx`)
   - Auto-initializes wallet when device connects
   - Triggers portfolio loading and sync operations
   - No onboarding gate before wallet operations

### Current Flow Problems

```
User starts app
├── Onboarding check (async)
├── Device scanning starts immediately (frontend_ready)
├── Device connects
│   ├── PIN dialog shows (before onboarding complete)
│   ├── Setup wizards launch
│   ├── Passphrase requests interrupt
│   └── VaultInterface shows
└── Onboarding wizard (if not yet shown) ← INTERRUPTED
```

## Technical Solution Approach

### 1. Onboarding Gate System

Create a centralized onboarding gate that prevents device interactions until onboarding is complete.

#### Core Components:

**A. OnboardingGateProvider** (New Context)
```tsx
interface OnboardingGateContext {
  isOnboardingComplete: boolean;
  allowDeviceInteractions: boolean;
  setOnboardingComplete: (complete: boolean) => void;
}
```

**B. Device Interaction Guard**
- Wrapper around all device event listeners
- Blocks device events when `allowDeviceInteractions = false`
- Queues critical device events for later processing

**C. Modified State Flow**
```
User starts app
├── Onboarding check
├── IF onboarding needed:
│   ├── Block device interactions
│   ├── Show onboarding wizard
│   └── Enable device interactions ONLY after complete
└── IF onboarding complete:
    └── Allow normal device flow
```

### 2. Implementation Strategy

#### Phase 1: Create Onboarding Gate

1. **Create OnboardingGateContext**
   - Track onboarding completion state
   - Provide device interaction blocking mechanism
   - Integrate with existing `useOnboardingState` hook

2. **Modify App.tsx Event Setup**
   - Wrap device event listeners with onboarding gate checks
   - Defer `frontend_ready` signal until onboarding complete
   - Queue device events during onboarding

3. **Update useDeviceInteraction Hook**
   - Add onboarding state dependency
   - Block PIN/passphrase dialogs during onboarding
   - Provide fallback handling for queued events

#### Phase 2: Device Manager Integration

1. **Modify DeviceUpdateManager**
   - Check onboarding state before showing dialogs
   - Defer device status handling until onboarding complete
   - Integrate with onboarding gate system

2. **Update WalletContext**
   - Block wallet initialization during onboarding
   - Defer portfolio loading until device interactions allowed
   - Prevent automatic sync operations

#### Phase 3: Event Queuing System

1. **Implement Event Queue**
   - Store critical device events during onboarding
   - Replay events after onboarding completion
   - Handle timeout and cleanup of stale events

2. **Priority Event Handling**
   - Define which events can be safely delayed
   - Implement immediate vs. deferred event categories
   - Handle device disconnection during onboarding

### 3. Code Changes Required

#### New Files:
- `/src/contexts/OnboardingGateContext.tsx` - Central gating mechanism
- `/src/hooks/useOnboardingGate.ts` - Hook for accessing gate state
- `/src/utils/deviceEventQueue.ts` - Event queuing system

#### Modified Files:

**App.tsx**
```tsx
// Wrap event listeners with onboarding gate
const { allowDeviceInteractions } = useOnboardingGate();

// Modify event listener setup
if (allowDeviceInteractions) {
  setupDeviceEventListeners();
} else {
  queueDeviceEvents();
}
```

**useDeviceInteraction.ts**
```tsx
// Add gating logic
const { allowDeviceInteractions } = useOnboardingGate();

if (!allowDeviceInteractions) {
  // Queue events instead of showing dialogs
  return;
}
```

**DeviceUpdateManager.tsx**
```tsx
// Check onboarding before device operations
const { isOnboardingComplete } = useOnboardingGate();

if (!isOnboardingComplete) {
  // Defer device status handling
  return null;
}
```

**OnboardingWizard.tsx**
```tsx
// Enable device interactions on completion
const handleComplete = async () => {
  await invoke("set_onboarding_completed");
  setOnboardingComplete(true); // Enable device interactions
  onComplete?.();
};
```

### 4. User Experience Flow

#### New Onboarding Flow:
```
1. App starts → Device scanning blocked
2. Onboarding wizard shows → User learns about KeepKey
3. User completes onboarding → Device interactions enabled
4. Device connects → Normal PIN/passphrase flow
5. VaultInterface shows → Full functionality available
```

#### Benefits:
- **Educational First**: Users learn before device interactions
- **No Interruptions**: Onboarding completes without dialogs
- **Seamless Transition**: Device flows work normally after onboarding
- **Backward Compatibility**: Existing users bypass onboarding

### 5. Edge Cases & Considerations

#### Device Already Connected
- Detect connected devices but don't interact until onboarding complete
- Show connection status without triggering authentication

#### Emergency Device Access
- Provide escape hatch for urgent device access during onboarding
- Show warning about skipping educational content

#### Multiple Devices
- Handle multiple device connections gracefully
- Queue events per device during onboarding

#### Backend State Consistency
- Ensure backend device state remains consistent during gating
- Handle device timeouts and reconnection scenarios

### 6. Testing Strategy

#### Unit Tests
- OnboardingGateContext state management
- Event queuing and replay functionality
- Device interaction blocking logic

#### Integration Tests
- Full onboarding flow with device connection
- Event queue behavior during onboarding
- Proper cleanup after onboarding completion

#### E2E Tests
- Complete user journey from first launch
- Device connection during onboarding
- Multiple device scenarios

### 7. Implementation Phases

#### Phase 1 (Core Gating) - 2-3 days
- OnboardingGateContext implementation
- Basic device interaction blocking
- App.tsx integration

#### Phase 2 (Component Integration) - 2-3 days
- DeviceUpdateManager modifications
- useDeviceInteraction updates
- WalletContext integration

#### Phase 3 (Event Queuing) - 1-2 days
- Event queue implementation
- Event replay after onboarding
- Edge case handling

#### Phase 4 (Testing & Polish) - 1-2 days
- Comprehensive testing
- UX refinements
- Documentation updates

**Total Estimated Effort: 6-10 days**

### 8. Success Criteria

- ✅ Onboarding completes without device dialog interruptions
- ✅ Device interactions work normally after onboarding
- ✅ Existing onboarded users are not affected
- ✅ Multiple device scenarios handled correctly
- ✅ Emergency access available if needed
- ✅ No regression in device functionality
- ✅ Clean separation between educational and functional flows

### 9. Risks & Mitigations

#### Risk: Backend Device State Drift
**Mitigation**: Implement device state refresh after onboarding completion

#### Risk: Event Queue Memory Leaks
**Mitigation**: Implement event expiration and cleanup mechanisms

#### Risk: User Confusion About Device Connection
**Mitigation**: Show clear status messages about onboarding requirements

#### Risk: Emergency Device Access Scenarios
**Mitigation**: Provide clear escape hatch with appropriate warnings

This plan provides a comprehensive approach to fixing the onboarding override issue while maintaining system stability and user experience quality.