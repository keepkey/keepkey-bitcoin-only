# KeepKey v1.0.3 Bootloader Update Guide

## Overview
The v1.0.3 bootloader is one of the earliest versions and requires special handling during updates. This guide covers common issues and solutions.

## Important Notes
- **The v1.0.3 bootloader requires manual button confirmation on the device**
- The device screen will show an "Upload" prompt that you must confirm
- The update process may take longer than with newer bootloaders

## Update Process

### 1. Device Must Be in Bootloader Mode
The device must already be in bootloader mode (showing "Connect to Begin" on screen). If it's not:
1. Disconnect the KeepKey
2. Hold down the button
3. While holding the button, connect the USB cable
4. Release the button when you see "Connect to Begin"

### 2. During the Update
When the update starts:
1. **Watch the device screen carefully**
2. When you see "Upload" appear on the screen, **press and hold the button**
3. Keep holding until the progress bar appears
4. The update will then proceed automatically

### 3. Common Issues

#### "No data received from device" Error
This usually means the device is waiting for button confirmation:
- Check the device screen for an "Upload" prompt
- Press and hold the button when you see it
- If nothing appears, try disconnecting and reconnecting in bootloader mode

#### USB Permission Errors
The app will automatically fall back to HID transport if USB permissions fail. This is normal and the update should still work.

#### Update Appears to Hang
The v1.0.3 bootloader may take up to 10 seconds to respond. Be patient and watch the device screen.

## After the Update
Once the bootloader update completes:
1. The device will automatically reboot
2. You'll need to update the firmware next
3. The device may show as v2.1.4 bootloader after successful update

## Troubleshooting
If the update fails repeatedly:
1. Try a different USB port (preferably USB 2.0)
2. Use a shorter USB cable
3. Ensure no other programs are accessing the device
4. On Windows, you may need to install the WinUSB driver using Zadig 