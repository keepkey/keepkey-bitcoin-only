## PIN and Passphrase Dialogs Audit (vault-v2)

Scope: Audit PIN-related and passphrase-related modals/flows in `projects/vault-v2/src`, identify triggers, overlaps, and inconsistencies, and establish a baseline for unification.

### Components (PIN)

- `components/PinUnlockDialog.tsx`: PIN unlock on startup/auth needs. Shown via `usePinUnlockDialog()`.
- `components/PinSetupDialog.tsx`: PIN setup with matrix UI.
- `components/PinCreationDialog.tsx`: PIN creation flow wrapper over `DevicePin` (create mode). Example shows standard dialog props and backdrop.
- `components/EnablePinDialog.tsx`: Instructional + launcher; opens `PinSetupDialog`.
- `components/PinRemovalDialog.tsx`: Remove PIN flow.
- `components/DevicePinDialog.tsx`: Generic device PIN dialog.
- Wizard usages:
  - `components/WalletCreationWizard/DevicePin.tsx`
  - `components/WalletCreationWizard/DevicePinHorizontal.tsx`
  - `components/WalletCreationWizard/RecoveryPin.tsx`
  - `components/SeedVerificationWizard/VerificationPin.tsx`
  - `components/OnboardingWizard/steps/Step2Pin.tsx`
  - `components/SetupWizard/steps/Step3Pin.tsx`
- `components/PinSettings.tsx`: Settings entry points to setup/remove PIN.

### Components (Passphrase)

- `components/SimplePassphraseModal.tsx`: Passphrase entry dialog used in event-driven flow.
- `components/PassphraseSettings.tsx`: Settings for passphrase behavior.

### Dialog registration and helpers

- `contexts/DialogContext.tsx`:
  - `usePinUnlockDialog()` shows `PinUnlockDialog` with `priority: 'critical'`, `persistent: true`.
  - `usePinCreationDialog()`, `usePinSetupDialog()`, `useEnablePinDialog()` similarly wired.
  - `usePassphraseDialog()` shows `SimplePassphraseModal` with `priority: 'high'`, `persistent: true`.

### Startup and event wiring (where they open)

- `contexts/WalletContext.tsx`:
  - Listens to `device:pin-request-triggered` and uses `usePinUnlockDialog()` to open a single PIN dialog per device. This is the canonical place for showing PIN unlock to avoid duplicates.

- `components/DeviceUpdateManager.tsx`:
  - Listens to `device:passphrase-unlock-needed`. When passphrase protection is enabled, it triggers a PIN request once via `invoke('trigger_pin_request', { deviceId })` to start the authentication flow, and ensures other dialogs are hidden.

- `hooks/useDeviceInteraction.ts`:
  - Listens to `device:awaiting_passphrase` and opens `SimplePassphraseModal` with correlation (`device_id`, `request_id`, `cache_allowed`). Handles submit via `invoke('passphrase_submit', ...)` and cancel via `invoke('passphrase_cancel', ...)`.

- `App.tsx` (legacy listener):
  - Listens to `passphrase_request` and calls `usePassphraseDialog().show(...)`. This appears redundant with `useDeviceInteraction.ts` and should be removed in favor of the centralized handler.

### Current pain points identified

- Duplicate/competing passphrase listeners:
  - Both `useDeviceInteraction.ts` and `App.tsx` listen for passphrase requests and can open a modal. This can race with the PIN dialog, leading to ordering issues and duplicate UI.

- Cross-dialog ordering and coupling:
  - PIN unlock is initiated from `WalletContext` on `device:pin-request-triggered` while passphrase is event-driven from `useDeviceInteraction`. These separate entry points make it easier to spawn the wrong dialog at the wrong time (e.g., passphrase popping before/over a PIN unlock dialog).

- Visual/style inconsistencies:
  - Dialog size, placement, motion, and backdrop vary across components. For example, `PinCreationDialog` uses `DialogRoot` with `size="full"`, `placement="center"`, `motionPreset="slide-in-bottom"`, and a darkened backdrop. Other dialogs may differ. Headers, close triggers, help text, and button variants are not guaranteed to be consistent.

- Multiple PIN-related implementations:
  - `PinUnlockDialog`, `DevicePinDialog`, wizard steps, and `PinSetupDialog` each implement similar UI/flows with slight variations.

### Uniformity baseline (recommended)

Apply consistently to all auth-related dialogs (PIN and passphrase):

- Dialog container:
  - `size: 'full'`, `placement: 'center'`, `motionPreset: 'slide-in-bottom'`.
  - Backdrop: `rgba(0, 0, 0, 0.8)`.
  - Persistent for auth-critical dialogs; user must explicitly close or complete.

- Header area:
  - Device avatar/icon + device label.
  - Title: "Unlock your device" for PIN, "Enter passphrase" for passphrase step.
  - Optional helper tooltip linking to docs.

- Content area:
  - PIN matrix component with clear instruction text when in PIN step.
  - Passphrase input with visibility toggle and optional "cache on device" when `cache_allowed` is true.

- Actions:
  - Primary button (Continue/Unlock), secondary (Cancel/Back) with consistent variants and spacing.

- Error states:
  - Non-blocking inline error messaging with retry guidance; critical errors may close dialog and surface global error.

### Consolidation opportunities

- deprecate direct usage of `usePassphraseDialog()` in `App.tsx` and any other callers; rely exclusively on a single flow manager.
- funnel both PIN and passphrase prompts through a single step-based dialog (see plan doc) to guarantee ordering: PIN → Passphrase (when required).
- reuse existing subcomponents (`PinUnlockDialog` content and `SimplePassphraseModal` content) inside a single wrapper to minimize risk while aligning styles.

### Acceptance targets for the unification work

- Only one auth dialog may be present per device at any time.
- When a device is passphrase-protected: PIN step is shown first; upon success, passphrase step appears; upon submit, dialog closes.
- When a device is PIN-only: only PIN step is shown; upon success, dialog closes.
- No passphrase dialog is ever opened from legacy `App.tsx` listener.
- Visual and interaction patterns match the uniformity baseline above across all PIN and passphrase dialogs and wizard steps.


