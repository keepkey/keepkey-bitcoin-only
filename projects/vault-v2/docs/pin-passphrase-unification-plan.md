## Plan: Unify PIN and Passphrase into a Single Step-Based Auth Dialog

Goal: Replace separate PIN and passphrase modals with one orchestrated, step-based dialog that opens on startup/auth-required events and guarantees correct ordering (PIN first, then passphrase when applicable), with uniform styling and predictable behavior.

### High-level design

- Introduce a new `AuthDialog` that manages a per-device authentication session with steps:
  - Step 1: PIN unlock (always triggered when device requests PIN)
  - Step 2: Passphrase entry (conditionally shown if device requires passphrase)
- `AuthDialog` is the only dialog handling authentication for a device. Other direct PIN/passphrase dialogs should not be opened concurrently.

### Sources of truth and triggers

- Continue to use `contexts/WalletContext.tsx` as the single place that reacts to `device:pin-request-triggered` and opens the auth dialog for the device.
- Consolidate passphrase request handling so that `useDeviceInteraction.ts` remains the centralized listener for `device:awaiting_passphrase` but instead of opening `SimplePassphraseModal`, it signals the active `AuthDialog` session to advance to the passphrase step.
- Remove the legacy `App.tsx` `passphrase_request` listener.

### Implementation outline

1) Create `components/AuthDialog.tsx` with internal state machine:
   - Props: `deviceId`, `initialStep` (default: 'pin'), optional callbacks.
   - Steps:
     - 'pin': render existing PIN content (reuse `PinUnlockDialog` contents or shared subcomponent).
     - 'passphrase': render existing passphrase content (reuse `SimplePassphraseModal` content or shared subcomponent).
   - Expose imperative API through dialog system via props or context to advance from 'pin' → 'passphrase'.

2) Dialog system integration (`contexts/DialogContext.tsx`):
   - Add `useAuthDialog()` that `show({ id: auth-${deviceId}, component: AuthDialog, props: { deviceId } })` with `priority: 'critical'`, `persistent: true`.
   - Deprecate direct `usePinUnlockDialog().show()` for unlock; instead call `useAuthDialog().show()`.

3) Event flow wiring:
   - In `WalletContext.tsx`, on `device:pin-request-triggered`, call `useAuthDialog().show({ deviceId })` if not already shown.
   - In `useDeviceInteraction.ts`, on `device:awaiting_passphrase`, do not open a new modal. Instead, locate the existing `auth-${device_id}` dialog and call an exported `advanceToPassphrase(device_id, request_id, cache_allowed)` method via the dialog props (or via a lightweight global registry inside `DialogContext`).
   - On passphrase submit/cancel, forward to existing `invoke('passphrase_submit' | 'passphrase_cancel', ...)` handlers.

4) Styling unification

- Use the uniformity baseline defined in `pin-passphrase-audit.md` for `AuthDialog` and retrofit subcomponents/wizard steps to match.
- Extract shared primitives (header, layout, actions) so `PinSetupDialog`, wizard PIN steps, and `SimplePassphraseModal` can adopt consistent visuals with minimal code changes.

5) Deprecations and cleanup

- Remove `App.tsx` `passphrase_request` listener.
- Update `WalletContext.tsx` to stop calling `usePinUnlockDialog()` for unlock; use `useAuthDialog()`.
- Keep `PinSetupDialog`/`EnablePinDialog`/`PinRemovalDialog` for settings and setup flows; they are not part of the runtime auth unlock flow, but align their styles.

### Rollout plan

- Phase 1 (Behind flag):
  - Implement `AuthDialog` and hook it up for devices requiring both PIN and passphrase.
  - Gate with an app config flag or dev build switch.

- Phase 2 (Default):
  - Remove legacy passphrase listener from `App.tsx`.
  - Switch `WalletContext` to `useAuthDialog()` for PIN unlock.

- Phase 3 (Cleanup):
  - Remove unused dialog wrappers and dead code.
  - Ensure all wizard steps and settings dialogs adhere to the unified style primitives.

### Acceptance criteria

- A single `AuthDialog` per device orchestrates both steps without spawning separate modals.
- No duplicate dialogs; dialog queue shows `auth-${deviceId}` only during authentication.
- Correct ordering and handoff between steps; errors are handled inline; closing the dialog cancels outstanding requests.
- Visual consistency across all related dialogs.

### Risks and mitigations

- Risk: Event race conditions between PIN and passphrase requests.
  - Mitigation: Centralize trigger in `WalletContext` and advance steps via `AuthDialog` API; ensure dialog `priority: 'critical'` and `persistent: true`.

- Risk: Backward compatibility with wizard/setup flows.
  - Mitigation: Keep setup flows independent; only unify unlock flows; extract shared primitives for look & feel.


