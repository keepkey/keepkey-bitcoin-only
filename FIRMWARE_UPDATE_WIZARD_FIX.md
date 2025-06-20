# Firmware Update Wizard Error Fix

## Problem
The firmware update wizard was failing to load with the error:
```
Failed to load dialog: firmware-update-wizard-33333038064737331C003700
undefined is not an object (evaluating 'currentStep.component')
```

## Root Cause Analysis

### Issue 1: Incorrect React.lazy() Import
In `DialogContext.tsx`, the firmware update wizard was imported incorrectly:

**❌ Before:**
```typescript
component: React.lazy(() => import('../components/FirmwareUpdateWizard/FirmwareUpdateWizard')),
```

**✅ After:**
```typescript
component: React.lazy(() => import('../components/FirmwareUpdateWizard/FirmwareUpdateWizard').then(m => ({ default: m.FirmwareUpdateWizard }))),
```

### Issue 2: Missing Safety Checks
The `FirmwareUpdateWizard` component had no protection against `currentStep` being undefined:

**❌ Before:**
```typescript
const currentStep = STEPS[currentStepIndex];
const StepComponent = currentStep.component;
```

**✅ After:**
```typescript
// Safety check to prevent undefined currentStep
const currentStep = STEPS[currentStepIndex] || STEPS[0];

// Additional safety check and logging
if (!STEPS[currentStepIndex]) {
  console.error(`🚨 [FirmwareUpdateWizard] Invalid currentStepIndex: ${currentStepIndex}, STEPS length: ${STEPS.length}`);
  console.error(`🚨 [FirmwareUpdateWizard] Resetting to step 0`);
  setCurrentStepIndex(0);
}

const StepComponent = currentStep?.component;

// Safety check - if no valid component, show error
if (!StepComponent || !currentStep) {
  return (
    <Box>
      <Text>Firmware Update Wizard Error</Text>
      <Text>Failed to load wizard step. Current step index: {currentStepIndex}</Text>
      <Button onClick={onClose}>Close</Button>
    </Box>
  );
}
```

## Files Modified

### 1. `projects/vault-v2/src/contexts/DialogContext.tsx`
- ✅ Fixed React.lazy() import for firmware update wizard to properly handle named export
- ✅ Import now matches the pattern used for bootloader update wizard

### 2. `projects/vault-v2/src/components/FirmwareUpdateWizard/FirmwareUpdateWizard.tsx`
- ✅ Added safety check to prevent `currentStep` from being undefined
- ✅ Added bounds checking for `currentStepIndex`
- ✅ Added error UI for invalid step states
- ✅ Added optional chaining for `currentStep?.component`

### 3. `projects/vault-v2/src/components/BootloaderUpdateWizard/BootloaderUpdateWizard.tsx`
- ✅ Applied same safety fixes to bootloader wizard for consistency
- ✅ Added safety check for `activeStep`
- ✅ Added error UI for invalid step states

## Expected Results

✅ **Firmware update wizard should now load successfully**
✅ **No more "undefined is not an object" errors**
✅ **Graceful error handling if step corruption occurs**
✅ **Consistent error handling across both wizards**
✅ **Better debugging information when issues occur**

## Testing

To test the fix:

1. **Connect a KeepKey device that needs firmware update**
2. **Trigger the firmware update wizard**
3. **Verify the wizard loads without errors**
4. **Check that all steps render correctly**

## Additional Improvements

The fixes also provide:
- 🔍 **Better error logging** for debugging wizard issues
- 🛡️ **Defensive programming** to prevent crashes from corrupted state
- 🔄 **Automatic recovery** by resetting to step 0 when needed
- 📊 **User-friendly error messages** instead of cryptic JavaScript errors

## Prevention

These changes prevent:
- Runtime errors from undefined step components
- App crashes from corrupted wizard state
- Poor user experience with cryptic error messages
- Similar issues in other lazy-loaded components 