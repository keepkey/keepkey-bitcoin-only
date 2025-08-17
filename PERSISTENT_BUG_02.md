# PERSISTENT BUG 02: Passphrase Submit Failure

## Bug Summary
Passphrase submission fails with "Unexpected message" error. Multiple PassphraseAck messages are being sent when device is not expecting them.

## Status
- **Severity**: Critical
- **First Reported**: Now
- **Current Attempt**: 1
- **Status**: FIXED
- **Resolution**: Added session tracking to prevent duplicate PassphraseAck submissions

## Problem Description
When attempting to submit a passphrase:
1. Device requests passphrase (`PassphraseRequest`)
2. User enters passphrase in dialog
3. First `PassphraseAck` is sent successfully
4. Device responds with `ButtonRequest`
5. `ButtonAck` is sent and user confirms on device
6. Operation times out after 30 seconds
7. User retries passphrase submission
8. Additional `PassphraseAck` messages are sent when device isn't expecting them
9. Device responds with `Failure: Unexpected message`

## Console Output Pattern
```
-> GetPublicKey
<- PassphraseRequest
🔐 Device requested passphrase for xpub
-> PassphraseAck
<- ButtonRequest
-> ButtonAck
Confirm action on device...
[30 second timeout]
ERROR: Device operation timed out
[User retries]
<- PublicKey [Device already sent the response!]
-> PassphraseAck [Sent again, but device isn't expecting it]
<- Failure: Unexpected message
```

## Technical Analysis

### What's Happening
1. **Successful Flow Started**: First PassphraseAck → ButtonRequest → ButtonAck flow is correct
2. **Timeout Issue**: Operation times out while waiting for user to confirm on device
3. **Device Already Responded**: Device actually sent `PublicKey` response after user confirmed
4. **Duplicate Submission**: Frontend allows resubmitting passphrase when it shouldn't
5. **Protocol Violation**: Sending PassphraseAck when device isn't in passphrase request state

### Root Cause
- The passphrase dialog doesn't properly track device state
- After sending PassphraseAck once, the dialog should either:
  - Close and wait for device confirmation
  - Prevent resubmission
  - Track that passphrase was already sent
- The 30-second timeout might be too short for user confirmation
- The system doesn't handle the case where device response arrives after timeout

## Key Files Involved
- Frontend passphrase dialog component
- `/src-tauri/src/commands.rs` - `send_passphrase` command
- Device communication layer

## Immediate Fix Needed

1. **Prevent Duplicate Submission**
   - Track if PassphraseAck was already sent for current session
   - Disable submit button after first submission
   - Clear session only when new PassphraseRequest arrives

2. **Handle Timeout Properly**
   - Increase timeout to 60 seconds for button confirmation
   - Check if device already responded before allowing retry
   - Clear pending state properly on timeout

3. **State Management**
   - Track passphrase request lifecycle
   - Prevent multiple PassphraseAck for same request
   - Properly reset state on new PassphraseRequest

## Current State
- Passphrase dialog allows multiple submissions
- Device rejects duplicate PassphraseAck messages
- User cannot complete passphrase-protected operations

## Solution Implemented

### Changes Made to SimplePassphraseModal.tsx

1. **Added Session State Tracking**
   - `hasSubmittedForSession`: Tracks if passphrase was already sent
   - `awaitingDeviceConfirmation`: Shows UI is waiting for device

2. **Prevented Duplicate Submissions**
   - Check `hasSubmittedForSession` before allowing submit
   - Show error message if user tries to submit again
   - Disable submit button after first submission

3. **Improved Error Handling**
   - Detect "Unexpected message" errors and prevent further attempts
   - Allow retry only for timeout errors
   - Clear messaging about device state

4. **Enhanced UI Feedback**
   - Show "Confirm on Device" when awaiting confirmation
   - Update button text to "Awaiting Device..." after submission
   - Disable input field while awaiting confirmation
   - Show appropriate status messages

5. **Session Reset Logic**
   - Reset session state when modal opens (new request)
   - Maintain state during the same passphrase request
   - Proper cleanup on modal close

## Result
Users can now:
- Submit passphrase once per request
- See clear feedback about device confirmation status
- Cannot accidentally send duplicate PassphraseAck messages
- Get appropriate error messages for different failure scenarios