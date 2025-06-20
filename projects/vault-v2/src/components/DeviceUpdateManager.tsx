import { useEffect, useState } from 'react'
import { BootloaderUpdateDialog } from './BootloaderUpdateDialog'
import { FirmwareUpdateDialog } from './FirmwareUpdateDialog'
import { WalletCreationWizard } from './WalletCreationWizard/WalletCreationWizard'
import { EnterBootloaderModeDialog } from './EnterBootloaderModeDialog'
import { PinUnlockDialog } from './PinUnlockDialog'
import type { DeviceStatus, DeviceFeatures } from '../types/device'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

interface DeviceUpdateManagerProps {
  // Optional callback when all updates/setup is complete
  onComplete?: () => void
}

export const DeviceUpdateManager = ({ onComplete }: DeviceUpdateManagerProps) => {
  const [deviceStatus, setDeviceStatus] = useState<DeviceStatus | null>(null)
  const [showEnterBootloaderMode, setShowEnterBootloaderMode] = useState(false)
  const [showBootloaderUpdate, setShowBootloaderUpdate] = useState(false)
  const [showFirmwareUpdate, setShowFirmwareUpdate] = useState(false)
  const [showWalletCreation, setShowWalletCreation] = useState(false)
  const [showPinUnlock, setShowPinUnlock] = useState(false)
  const [isProcessing, setIsProcessing] = useState(false)
  const [connectedDeviceId, setConnectedDeviceId] = useState<string | null>(null)
  const [retryCount, setRetryCount] = useState(0)

  // Function to try getting device status via command when events fail
  const tryGetDeviceStatus = async (deviceId: string, attempt = 1) => {
    const maxAttempts = 3
    console.log(`Attempting to get device status for ${deviceId} (attempt ${attempt}/${maxAttempts})`)
    
    try {
      const status = await invoke<DeviceStatus | null>('get_device_status', { deviceId })
      if (status) {
        console.log('Successfully got device status via command:', status)
        setDeviceStatus(status)
        handleDeviceStatus(status)
        return true
      } else {
        console.log('No device status returned')
        return false
      }
    } catch (error) {
      console.error(`Failed to get device status (attempt ${attempt}):`, error)
      
      if (attempt < maxAttempts) {
        const delay = Math.min(1000 * Math.pow(2, attempt - 1), 5000) // Exponential backoff, max 5s
        console.log(`Retrying in ${delay}ms...`)
        setTimeout(() => {
          tryGetDeviceStatus(deviceId, attempt + 1)
        }, delay)
      } else {
        console.error('Max attempts reached, giving up on getting device status')
      }
      return false
    }
  }

  // Function to handle device status and determine which dialog to show
  const handleDeviceStatus = (status: DeviceStatus) => {
    console.log('🔧 DeviceUpdateManager: Handling device status:', status)
    console.log('🔧 DeviceUpdateManager: Status needs_initialization:', status.needsInitialization)
    console.log('🔧 DeviceUpdateManager: Status needs_firmware_update:', status.needsFirmwareUpdate)
    console.log('🔧 DeviceUpdateManager: Status needs_bootloader_update:', status.needsBootloaderUpdate)
    console.log('🔧 DeviceUpdateManager: Status needs_pin_unlock:', status.needsPinUnlock)
    
    // Check if device is in bootloader mode - handle both field formats from backend
    const isInBootloaderMode = status.features?.bootloader_mode || status.features?.bootloaderMode || false
    console.log('🔧 Bootloader mode check:', {
      bootloader_mode: status.features?.bootloader_mode,
      bootloaderMode: status.features?.bootloaderMode,
      isInBootloaderMode,
      needsBootloaderUpdate: status.needsBootloaderUpdate,
      hasBootloaderCheck: !!status.bootloaderCheck
    })
    
    // Determine which dialog to show based on priority
    if (status.needsBootloaderUpdate && status.bootloaderCheck) {
      if (isInBootloaderMode) {
        // Device needs bootloader update AND is in bootloader mode -> show update dialog
        console.log('Device needs bootloader update and is in bootloader mode')
        setShowEnterBootloaderMode(false)
        setShowBootloaderUpdate(true)
        setShowFirmwareUpdate(false)
        setShowWalletCreation(false)
      } else {
        // Device needs bootloader update but NOT in bootloader mode -> show enter bootloader mode dialog
        console.log('Device needs bootloader update but not in bootloader mode')
        setShowEnterBootloaderMode(true)
        setShowBootloaderUpdate(false)
        setShowFirmwareUpdate(false)
        setShowWalletCreation(false)
      }
    } else if (status.needsFirmwareUpdate && status.firmwareCheck) {
      console.log('Device needs firmware update')
      setShowEnterBootloaderMode(false)
      setShowBootloaderUpdate(false)
      setShowFirmwareUpdate(true)
      setShowWalletCreation(false)
    } else if (status.needsInitialization) {
      // Check if recovery is in progress - if so, don't interfere
      if ((window as any).KEEPKEY_RECOVERY_IN_PROGRESS) {
        console.log('🛡️ DeviceUpdateManager: Recovery in progress - IGNORING initialization request')
        console.log('🛡️ DeviceUpdateManager: Keeping current state to protect recovery')
        return; // Don't change any state during recovery
      }
      
      console.log('🔧 DeviceUpdateManager: Device needs initialization - SHOULD SHOW ONBOARDING WIZARD')
      console.log('🔧 DeviceUpdateManager: Setting showWalletCreation = true')
      setShowEnterBootloaderMode(false)
      setShowBootloaderUpdate(false)
      setShowFirmwareUpdate(false)
      setShowWalletCreation(true)
    } else if (status.needsPinUnlock) {
      // Device is initialized but locked with PIN - show PIN unlock dialog
      console.log('🔒 DeviceUpdateManager: Device needs PIN unlock - showing PIN dialog')
      setShowEnterBootloaderMode(false)
      setShowBootloaderUpdate(false)
      setShowFirmwareUpdate(false)
      setShowWalletCreation(false)
      setShowPinUnlock(true)
    } else {
      // Device is ready
      console.log('🔧 DeviceUpdateManager: Device is ready, no updates needed')
      console.log('🔧 DeviceUpdateManager: Calling onComplete() - this will show VaultInterface')
      setShowEnterBootloaderMode(false)
      setShowBootloaderUpdate(false)
      setShowFirmwareUpdate(false)
      setShowWalletCreation(false)
      setShowPinUnlock(false)
      onComplete?.()
    }
  }

  useEffect(() => {
    let featuresUnsubscribe: Promise<() => void> | null = null
    let connectedUnsubscribe: Promise<() => void> | null = null
    let timeoutId: NodeJS.Timeout | null = null

    const setupListeners = async () => {
      console.log('DeviceUpdateManager: Setting up event listeners...')
      
      // Listen for device features updates which include status (primary method)
      featuresUnsubscribe = listen<{
        deviceId: string
        features: DeviceFeatures
        status: DeviceStatus
      }>('device:features-updated', (event) => {
        console.log('🔧 DeviceUpdateManager: Device features updated event received:', event.payload)
        const { status } = event.payload
        console.log('🔧 DeviceUpdateManager: Extracted status from event:', status)
        
        // Check if recovery is in progress - if so, be very careful about state changes
        if ((window as any).KEEPKEY_RECOVERY_IN_PROGRESS) {
          console.log('🛡️ DeviceUpdateManager: Recovery in progress - handling features event carefully')
          // Still update device status (for recovery to work) but don't trigger UI changes
          setDeviceStatus(status)
          setConnectedDeviceId(status.deviceId)
          setRetryCount(0)
          // DO NOT call handleDeviceStatus during recovery to prevent UI conflicts
          return;
        }
        
        setDeviceStatus(status)
        setConnectedDeviceId(status.deviceId)
        setRetryCount(0) // Reset retry count on successful event
        handleDeviceStatus(status)
      })

      // Listen for basic device connected events as fallback
      connectedUnsubscribe = listen<{
        unique_id: string
        name: string
        vid: number
        pid: number
        manufacturer?: string
        product?: string
        serial_number?: string
        is_keepkey: boolean
      }>('device:connected', (event) => {
        const device = event.payload
        console.log('Device connected event received (fallback):', device)
        
        if (device.is_keepkey) {
          setConnectedDeviceId(device.unique_id)
          
          // Set a timeout to try getting device status if features event doesn't come
          if (timeoutId) clearTimeout(timeoutId)
          timeoutId = setTimeout(() => {
            console.log('Features event timeout, trying direct device status call...')
            tryGetDeviceStatus(device.unique_id)
          }, 3000) // Wait 3 seconds for features event before trying fallback
        }
      })

      // Listen for device access errors
      const accessErrorUnsubscribe = listen<{
        deviceId: string
        error: string
        errorType: string
        status: string
      }>('device:access-error', (event) => {
        console.log('Device access error received:', event.payload)
        // Clear any pending dialogs when there's an access error
        setShowBootloaderUpdate(false)
        setShowFirmwareUpdate(false)
        setShowWalletCreation(false)
        setDeviceStatus(null)
        setConnectedDeviceId(null)
      })

      // Listen for PIN unlock needed events
      const pinUnlockUnsubscribe = listen<{
        deviceId: string
        features: DeviceFeatures
        status: DeviceStatus
        needsPinUnlock: boolean
      }>('device:pin-unlock-needed', (event) => {
        console.log('🔒 DeviceUpdateManager: PIN unlock needed event received:', event.payload)
        const { status } = event.payload
        
        // Show PIN unlock dialog instead of initialization
        console.log('🔒 DeviceUpdateManager: Setting showPinUnlock = true')
        setDeviceStatus(status)
        setConnectedDeviceId(status.deviceId)
        setShowEnterBootloaderMode(false)
        setShowBootloaderUpdate(false)
        setShowFirmwareUpdate(false)
        setShowWalletCreation(false)
        setShowPinUnlock(true)
      })

      // Listen for device disconnection
      const disconnectedUnsubscribe = listen<string>('device:disconnected', (event) => {
        console.log('Device disconnected:', event.payload)
        
        // Check if recovery is in progress - if so, ignore disconnection events
        if ((window as any).KEEPKEY_RECOVERY_IN_PROGRESS) {
          console.log('🛡️ DeviceUpdateManager: Recovery in progress - IGNORING disconnection event')
          console.log('🛡️ DeviceUpdateManager: Keeping current state to protect recovery')
          return; // Don't change state during recovery
        }
        
        // Clear all state when device disconnects (only if not in recovery)
        setDeviceStatus(null)
        setConnectedDeviceId(null)
        setShowBootloaderUpdate(false)
        setShowFirmwareUpdate(false)
        setShowWalletCreation(false)
        setShowPinUnlock(false)
        setRetryCount(0)
        if (timeoutId) clearTimeout(timeoutId)
      })

      // Frontend ready signal is now sent by App.tsx during initial setup

      return async () => {
        if (featuresUnsubscribe) (await featuresUnsubscribe)()
        if (connectedUnsubscribe) (await connectedUnsubscribe)()
        ;(await accessErrorUnsubscribe)()
        ;(await pinUnlockUnsubscribe)()
        ;(await disconnectedUnsubscribe)()
        if (timeoutId) clearTimeout(timeoutId)
      }
    }

    setupListeners()

    return () => {
      // Cleanup function will be called automatically
      if (timeoutId) clearTimeout(timeoutId)
    }
  }, [onComplete])

  const handleFirmwareUpdate = async () => {
    setIsProcessing(true)
    try {
      // Update firmware using our implemented Tauri command
      await invoke('update_device_firmware', { 
        deviceId: deviceStatus?.deviceId,
        targetVersion: deviceStatus?.firmwareCheck?.latestVersion || ''
      })
      
      // After successful update, check if initialization is needed
      setShowFirmwareUpdate(false)
    } catch (error) {
      console.error('Firmware update failed:', error)
      // TODO: Show error dialog
    } finally {
      setIsProcessing(false)
    }
  }

  const handleFirmwareSkip = () => {
    setShowFirmwareUpdate(false)
    
    // Check if we need to show wallet creation
    if (deviceStatus?.needsInitialization) {
      setShowWalletCreation(true)
    } else {
      onComplete?.()
    }
  }

  const handleFirmwareRemindLater = () => {
    // TODO: Store reminder preference
    setShowFirmwareUpdate(false)
    
    // Continue to next step
    if (deviceStatus?.needsInitialization) {
      setShowWalletCreation(true)
    } else {
      onComplete?.()
    }
  }

  const handleWalletCreationComplete = () => {
    setShowWalletCreation(false)
    onComplete?.()
  }

  const handleEnterBootloaderModeClose = () => {
    setShowEnterBootloaderMode(false)
    // Don't call onComplete here - wait for user to actually enter bootloader mode
  }

  const handlePinUnlocked = () => {
    console.log('🔒 PIN unlock successful, device is now unlocked')
    setShowPinUnlock(false)
    // Device should now be ready to use
    onComplete?.()
  }

  const handlePinUnlockClose = () => {
    setShowPinUnlock(false)
    // Don't call onComplete - user cancelled PIN entry
  }

  if (!deviceStatus) {
    console.log('🔧 DeviceUpdateManager: No deviceStatus, returning null')
    return null
  }

  console.log('🔧 DeviceUpdateManager: Rendering with state:', {
    showWalletCreation,
    showFirmwareUpdate,
    showBootloaderUpdate,
    showEnterBootloaderMode,
    showPinUnlock,
    deviceStatus: deviceStatus?.needsInitialization
  })

  return (
    <>
      {showEnterBootloaderMode && deviceStatus.bootloaderCheck && deviceStatus.deviceId && (
        <EnterBootloaderModeDialog
          isOpen={showEnterBootloaderMode}
          bootloaderCheck={deviceStatus.bootloaderCheck}
          deviceId={deviceStatus.deviceId}
          onClose={handleEnterBootloaderModeClose}
        />
      )}

      {showBootloaderUpdate && deviceStatus.bootloaderCheck && deviceStatus.deviceId && (
        <BootloaderUpdateDialog
          isOpen={showBootloaderUpdate}
          bootloaderCheck={deviceStatus.bootloaderCheck}
          deviceId={deviceStatus.deviceId}
          onUpdateComplete={() => {
            setShowBootloaderUpdate(false)
            // The device will restart and emit new features
          }}
        />
      )}

      {showFirmwareUpdate && deviceStatus.firmwareCheck && (
        <FirmwareUpdateDialog
          isOpen={showFirmwareUpdate}
          firmwareCheck={deviceStatus.firmwareCheck}
          onUpdateStart={handleFirmwareUpdate}
          onSkip={handleFirmwareSkip}
          onRemindLater={handleFirmwareRemindLater}
          onClose={() => setShowFirmwareUpdate(false)}
          isLoading={isProcessing}
        />
      )}

      {showWalletCreation && deviceStatus.deviceId && (
        <WalletCreationWizard
          deviceId={deviceStatus.deviceId}
          onComplete={handleWalletCreationComplete}
          onClose={() => setShowWalletCreation(false)}
        />
      )}

      {showPinUnlock && deviceStatus.deviceId && (
        <PinUnlockDialog
          isOpen={showPinUnlock}
          deviceId={deviceStatus.deviceId}
          onUnlocked={handlePinUnlocked}
          onClose={handlePinUnlockClose}
        />
      )}
    </>
  )
} 