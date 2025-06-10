KeepKey onboarding flow


states:

no device:
(ask user to connect device)
(offer troubleshooter)

Device Connected (identified by VID: 0x2b24, PID: 0x0002 - this PID might be the same for normal and updater modes):

    Attempt to send GetFeatures message:

    If GetFeatures successful AND Features.bootloader_mode == false (Normal Mode):
        Detect application firmware version (from Features.major_version, etc.).
        Detect bootloader version (if available from Features, otherwise may be unknown in this state).
        Store these versions (especially application firmware version) in memory (WizardCtx).
        If app firmware version is old or unknown:
            Offer update: Guide user to ensure device is (or will be put) into updater mode.

    If GetFeatures successful AND Features.bootloader_mode == true (Updater/Bootloader Mode):
        The device is confirmed to be in the correct state for updates.
        Versions reported by GetFeatures in this state likely pertain to the bootloader itself.
        Rely on remembered application firmware version (if any) from WizardCtx to decide if an app firmware update is appropriate.
        Offer bootloader update (e.g., to latest).
        Offer application firmware update (e.g., to latest or custom).

    If GetFeatures fails (e.g., cannot connect to USB transport, or device does not respond as expected):
        Treat as "NoDevice" or an "Unresponsive/Unknown KeepKey State".
        Log the error.
        Guide user to check connection, potentially try replugging.

Key Learnings & Detection Strategy:

- The KeepKey device may use the same USB Product ID (PID) (e.g., 0x0002 alongside VID 0x2b24) for both its normal operational mode and its updater/bootloader mode.
- PID alone is NOT sufficient to distinguish modes.
- To determine the mode:
    1. Connect to the device interface (VID/PID).
    2. Send a GetFeatures command.
    3. Check the `bootloader_mode` boolean flag in the `Features` response.
        - `true`: Device is in updater/bootloader state.
        - `false`: Device is in normal operational mode.
- When `bootloader_mode` is `true`, firmware versions reported by `GetFeatures` likely pertain to the bootloader. Application firmware versions should be read and remembered when `bootloader_mode` is `false`.
- The existing `kkcli firmware-update` command likely assumes the user manually puts the device into bootloader mode first.

Original Issue Note (still relevant):
"so we have an issue, in updater mode we DO NOT know the firmware version
or even the bootloader version [of the application firmware].
We need to save these things into memory and use the memory to help user decide what to do next"
This is addressed by reading versions in normal mode and storing them in WizardCtx.

