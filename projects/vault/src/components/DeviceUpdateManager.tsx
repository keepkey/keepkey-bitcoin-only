import { useEffect, useState } from 'react'
import { BootloaderUpdateDialog } from './BootloaderUpdateDialog'
import { FirmwareUpdateDialog } from './FirmwareUpdateDialog'
import { OnboardingWizard } from './OnboardingWizard/OnboardingWizard'
import type { DeviceStatus, DeviceFeatures } from '../types/device'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

interface DeviceUpdateManagerProps {
  // Optional callback when all updates/setup is complete
  onComplete?: () => void
}

export const DeviceUpdateManager = ({ onComplete }: DeviceUpdateManagerProps) => {
  const [deviceStatus, setDeviceStatus] = useState<DeviceStatus | null>(null)
  const [showBootloaderUpdate, setShowBootloaderUpdate] = useState(false)
  const [showFirmwareUpdate, setShowFirmwareUpdate] = useState(false)
  const [showWalletCreation, setShowWalletCreation] = useState(false)
  const [isProcessing, setIsProcessing] = useState(false)

  useEffect(() => {
    // Listen for device features updates which include status
    const unsubscribe = listen<{
      deviceId: string
      features: DeviceFeatures
      status: DeviceStatus
    }>('device:features-updated', (event) => {
      const { status } = event.payload
      setDeviceStatus(status)
      
      // Determine which dialog to show based on priority
      if (status.needsBootloaderUpdate && status.bootloaderCheck) {
        setShowBootloaderUpdate(true)
        setShowFirmwareUpdate(false)
        setShowWalletCreation(false)
      } else if (status.needsFirmwareUpdate && status.firmwareCheck) {
        setShowBootloaderUpdate(false)
        setShowFirmwareUpdate(true)
        setShowWalletCreation(false)
      } else if (status.needsInitialization) {
        setShowBootloaderUpdate(false)
        setShowFirmwareUpdate(false)
        setShowWalletCreation(true)
      } else {
        // Device is ready
        setShowBootloaderUpdate(false)
        setShowFirmwareUpdate(false)
        setShowWalletCreation(false)
        onComplete?.()
      }
    })

    return () => {
      unsubscribe.then(fn => fn())
    }
  }, [onComplete])



  const handleFirmwareUpdate = async () => {
    setIsProcessing(true)
    try {
      // Update firmware using our implemented Tauri command
      await invoke('update_device_firmware', { 
        deviceId: deviceStatus?.deviceId,
        targetVersion: deviceStatus?.firmwareCheck?.targetVersion || ''
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

  if (!deviceStatus) return null

  return (
    <>
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

      {showWalletCreation && (
        <OnboardingWizard
          onComplete={handleWalletCreationComplete}
          onClose={() => setShowWalletCreation(false)}
        />
      )}
    </>
  )
} 