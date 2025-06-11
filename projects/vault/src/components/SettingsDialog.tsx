import { Tabs, VStack, Text, Button, Icon, Box } from '@chakra-ui/react'
import { 
  DialogRoot,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogBody,
  DialogCloseTrigger
} from './ui/dialog'
import { LuSettings, LuMonitor, LuCpu, LuNetwork } from 'react-icons/lu'
import { FaCog } from 'react-icons/fa'
import { useState, useEffect } from 'react'
import { VaultSetup } from './VaultSetup'
import { KeepKeyDeviceList } from './KeepKeyDeviceList'
import { BootloaderUpdateDialog } from './BootloaderUpdateDialog'
import { FirmwareUpdateDialog } from './FirmwareUpdateDialog'
import SeedVerificationWizard from './SeedVerificationWizard/SeedVerificationWizard'
import type { DeviceStatus } from '../types/device'
import { invoke } from '@tauri-apps/api/core'
import holdAndConnectSvg from '../assets/svg/hold-and-connect.svg'
import { useFirmwareUpdateWizard, useWalletCreationWizard } from '../contexts/DialogContext'

interface SettingsDialogProps {
  isOpen: boolean
  onClose: () => void
}

export const SettingsDialog = ({ isOpen, onClose }: SettingsDialogProps) => {
  const [selectedDeviceId, setSelectedDeviceId] = useState<string | null>(null)
  const [showBootloaderUpdate, setShowBootloaderUpdate] = useState(false)
  const [showFirmwareUpdate, setShowFirmwareUpdate] = useState(false)
  const [deviceStatus, setDeviceStatus] = useState<DeviceStatus | null>(null)
  
  // Seed verification wizard state
  const [verificationWizardOpen, setVerificationWizardOpen] = useState(false)
  const [verificationDeviceId, setVerificationDeviceId] = useState<string | null>(null)
  const [verificationDeviceLabel, setVerificationDeviceLabel] = useState<string | null>(null)
  
  const firmwareWizard = useFirmwareUpdateWizard()
  const walletCreationWizard = useWalletCreationWizard()
  
  // Define fetchDeviceStatus function outside of useEffect
  const fetchDeviceStatus = async (deviceId = selectedDeviceId) => {
    if (deviceId) {
      try {
        console.log('Fetching device status for deviceId:', deviceId)
        const status = await invoke<DeviceStatus | null>('get_device_status', { 
          deviceId: deviceId 
        })
        if (status) {
          console.log('Received device status:', status)
          setDeviceStatus(status)
        } else {
          console.warn('Received null device status')
        }
      } catch (error) {
        console.error('Failed to get device status:', error)
      }
    } else {
      console.warn('No deviceId provided for fetchDeviceStatus')
    }
  }
  
  // Fetch device status when a device is selected
  useEffect(() => {
    fetchDeviceStatus()
  }, [selectedDeviceId])

  // Remove firmware version fetching - backend handles this

  const handleBootloaderUpdate = async (deviceId: string) => {
    console.log('handleBootloaderUpdate called with deviceId:', deviceId)
    setSelectedDeviceId(deviceId)
    
    try {
      // Directly call the bootloader update command with a target version
      // Using "2.1.0" as the target version (from device_update.rs LATEST_BOOTLOADER_VERSION)
      console.log('Calling update_device_bootloader with deviceId:', deviceId)
      
      // Call the backend command to update the bootloader
      // This will internally create a high-priority blocking action
      const result = await invoke('update_device_bootloader', { 
        deviceId, 
        targetVersion: "2.1.0" 
      }) as boolean
      
      console.log('Bootloader update result:', result)
      
      if (result) {
        // Show success message and close settings dialog
        alert('Bootloader update initiated. Please follow the instructions on your device.')
        // Close the settings dialog
        onClose()
      } else {
        throw new Error("Bootloader update failed")
      }
    } catch (error) {
      console.error('Failed to start bootloader update:', error)
      
      // Show error message
      alert(`Failed to start bootloader update: ${error}`)
      
      // Fallback to direct dialog if command fails and close settings dialog
      setShowBootloaderUpdate(true)
      // Close the settings dialog to prevent multiple dialogs
      onClose()
    }
  }

  const handleFirmwareUpdate = async (deviceId: string) => {
    console.log('handleFirmwareUpdate called with deviceId:', deviceId)
    setSelectedDeviceId(deviceId)
    
    try {
      // Get device info to find current firmware version
      const deviceInfo = await invoke('get_device_info_by_id', { deviceId }) as any
      console.log('Device info for firmware update:', deviceInfo)
      
      const currentVersion = deviceInfo?.version || deviceInfo?.features?.version || "2.1.4"
      const targetVersion = "7.10.0" // From releases.json
      
      console.log(`Opening firmware update wizard: ${currentVersion} -> ${targetVersion}`)
      
      // Close settings dialog first
      onClose()
      
      // Show the firmware update wizard directly
      firmwareWizard.show({
        deviceId,
        currentVersion,
        targetVersion,
        onWizardClose: () => {
          console.log('Firmware update wizard closed')
        },
        onComplete: (success: boolean, updatedDeviceId: string) => {
          console.log('Firmware update completed:', success, updatedDeviceId)
        }
      })
    } catch (error) {
      console.error('Failed to open firmware update wizard:', error)
      alert(`Failed to open firmware update wizard: ${error}`)
    }
  }

  const handleCreateWallet = async (deviceId: string) => {
    console.log('🚀 [SettingsDialog] handleCreateWallet called with deviceId:', deviceId)
    setSelectedDeviceId(deviceId)
    
    try {
      console.log('🚀 [SettingsDialog] Opening wallet creation wizard for device:', deviceId)
      console.log('🚀 [SettingsDialog] Current walletCreationWizard:', walletCreationWizard)
      
      // Close settings dialog first
      console.log('🚀 [SettingsDialog] Closing settings dialog...')
      onClose()
      
      // Add a small delay to ensure settings dialog closes first
      setTimeout(() => {
        console.log('🚀 [SettingsDialog] Calling walletCreationWizard.show()...')
        
        // Show the wallet creation wizard using DialogContext
        walletCreationWizard.show({
          deviceId,
          onWizardClose: () => {
            console.log('🚀 [SettingsDialog] Wallet creation wizard closed')
          },
          onWizardComplete: () => {
            console.log('🚀 [SettingsDialog] Wallet creation wizard completed for device:', deviceId)
          }
        })
        
        console.log('🚀 [SettingsDialog] walletCreationWizard.show() called successfully')
      }, 100)
      
    } catch (error) {
      console.error('🚀 [SettingsDialog] Failed to open wallet creation wizard:', error)
      alert(`Failed to open wallet creation wizard: ${error}`)
    }
  }

  const handleVerifySeed = (deviceId: string, deviceLabel?: string) => {
    console.log('🔍 [SettingsDialog] handleVerifySeed called with deviceId:', deviceId)
    
    // Store device info for verification wizard
    setVerificationDeviceId(deviceId)
    setVerificationDeviceLabel(deviceLabel || null)
    
    // Close settings dialog first
    console.log('🔍 [SettingsDialog] Closing settings dialog...')
    onClose()
    
    // Add a small delay to ensure settings dialog closes first
    setTimeout(() => {
      console.log('🔍 [SettingsDialog] Opening verification wizard...')
      setVerificationWizardOpen(true)
    }, 100)
  }

  const handleVerificationClose = () => {
    console.log('🔍 [SettingsDialog] Seed verification wizard closed')
    setVerificationWizardOpen(false)
    setVerificationDeviceId(null)
    setVerificationDeviceLabel(null)
  }
  
  return (
    <>
      <DialogRoot open={isOpen} onOpenChange={({ open }) => !open && onClose()}>
        <DialogContent 
          width="600px"
          height="700px"
          bg="gray.900" 
          color="white"
          border="2px solid"
          borderColor="gray.600"
          boxShadow="0 25px 50px -12px rgba(0, 0, 0, 0.8), 0 8px 16px -8px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.1)"
          borderRadius="xl"
          position="relative"
          _before={{
            content: '""',
            position: "absolute",
            top: "-1px",
            left: "-1px",
            right: "-1px",
            bottom: "-1px",
            background: "linear-gradient(135deg, rgba(255, 255, 255, 0.2), rgba(255, 255, 255, 0.05), rgba(255, 255, 255, 0.1))",
            borderRadius: "xl",
            zIndex: -1,
          }}
        >
          <DialogHeader borderBottomWidth="1px" borderColor="gray.700" pb={4}>
            <DialogTitle color="white">Settings</DialogTitle>
            <DialogCloseTrigger color="gray.400" _hover={{ color: "white" }} />
          </DialogHeader>
          
          <DialogBody p={6} flex="1" overflowY="auto">
            <Tabs.Root defaultValue="general">
              <Tabs.List bg="gray.800" borderRadius="md" p={1} mb={6}>
                <Tabs.Trigger 
                  value="general"
                  flex="1"
                  gap={2}
                  color="gray.400"
                  _selected={{ bg: "gray.700", color: "white" }}
                  _hover={{ color: "white" }}
                >
                  <LuSettings size={16} />
                  General
                </Tabs.Trigger>
                <Tabs.Trigger 
                  value="app"
                  flex="1"
                  gap={2}
                  color="gray.400"
                  _selected={{ bg: "gray.700", color: "white" }}
                  _hover={{ color: "white" }}
                >
                  <LuMonitor size={16} />
                  App
                </Tabs.Trigger>
                <Tabs.Trigger 
                  value="keepkey"
                  flex="1"
                  gap={2}
                  color="gray.400"
                  _selected={{ bg: "gray.700", color: "white" }}
                  _hover={{ color: "white" }}
                >
                  <LuCpu size={16} />
                  KeepKey
                </Tabs.Trigger>
                <Tabs.Trigger 
                  value="mcp"
                  flex="1"
                  gap={2}
                  color="gray.400"
                  _selected={{ bg: "gray.700", color: "white" }}
                  _hover={{ color: "white" }}
                >
                  <LuNetwork size={16} />
                  MCP
                </Tabs.Trigger>
              </Tabs.List>

              <Tabs.Content value="general" minHeight="400px" overflowY="auto">
                <VStack align="stretch" gap={4}>
                  <VaultSetup />
                </VStack>
              </Tabs.Content>

              <Tabs.Content value="app" minHeight="400px" overflowY="auto">
                <VStack align="stretch" gap={4}>
                  <Text color="gray.300">App settings content coming soon...</Text>
                </VStack>
              </Tabs.Content>

              <Tabs.Content value="keepkey" minHeight="400px" overflowY="auto">
                <VStack align="stretch" gap={4}>
                  <KeepKeyDeviceList 
                    onBootloaderUpdate={handleBootloaderUpdate}
                    onFirmwareUpdate={handleFirmwareUpdate}
                    onCreateWallet={handleCreateWallet}
                    onVerifySeed={handleVerifySeed}
                  />
                </VStack>
              </Tabs.Content>

              <Tabs.Content value="mcp" minHeight="400px" overflowY="auto">
                <VStack align="stretch" gap={4}>
                  <Text color="gray.300">MCP settings content coming soon...</Text>
                </VStack>
              </Tabs.Content>
            </Tabs.Root>
          </DialogBody>
        </DialogContent>
      </DialogRoot>

      {/* Update Dialogs - Only shown when settings dialog is closed */}
      {showBootloaderUpdate && !isOpen && (
        deviceStatus?.bootloaderCheck ? (
          <BootloaderUpdateDialog
            isOpen={showBootloaderUpdate}
            bootloaderCheck={deviceStatus.bootloaderCheck}
            deviceId={selectedDeviceId || ''}
            onUpdateComplete={() => {
              console.log('Bootloader update completed for device:', selectedDeviceId)
              setShowBootloaderUpdate(false)
            }}
          />
        ) : (
          <DialogRoot open={showBootloaderUpdate} onOpenChange={() => setShowBootloaderUpdate(false)}>
            <DialogContent maxW="md" bg="gray.900" color="white" borderColor="orange.600" borderWidth="2px">
              <DialogHeader borderBottomWidth="1px" borderColor="gray.700" pb={4}>
                <DialogTitle color="white">Device Not In Updater Mode</DialogTitle>
                <DialogCloseTrigger color="gray.400" _hover={{ color: "white" }} />
              </DialogHeader>
              <DialogBody py={6}>
                <VStack align="stretch" gap={4}>
                  <Text fontWeight="bold">Your KeepKey needs to be in Bootloader Mode to update the bootloader.</Text>
                  
                  <Box display="flex" justifyContent="center" py={4}>
                    <img 
                      src={holdAndConnectSvg} 
                      alt="Hold button while connecting device" 
                      style={{ maxWidth: '240px', height: 'auto' }} 
                    />
                  </Box>
                  
                  <VStack align="stretch" gap={2} bg="gray.800" p={4} borderRadius="md">
                    <Text fontWeight="semibold">How to enter Bootloader Mode:</Text>
                    <Text fontSize="sm">1. Unplug your KeepKey</Text>
                    <Text fontSize="sm">2. Hold down the button on your device</Text>
                    <Text fontSize="sm">3. While holding the button, plug in your KeepKey</Text>
                    <Text fontSize="sm">4. Continue holding until you see "BOOTLOADER MODE" on the screen</Text>
                    <Text fontSize="sm">5. Release the button and try the update again</Text>
                  </VStack>
                  
                  <Text fontSize="sm" color="gray.400">Device ID: {selectedDeviceId || 'Unknown'}</Text>
                </VStack>
              </DialogBody>
              <Box borderTopWidth="1px" borderColor="gray.700" pt={4} mt={4}>
                <Button colorScheme="orange" onClick={() => setShowBootloaderUpdate(false)} width="full">
                  I'll Try Again Later
                </Button>
              </Box>
            </DialogContent>
          </DialogRoot>
        )
      )}

      {showFirmwareUpdate && !isOpen && deviceStatus?.firmwareCheck && (
        <FirmwareUpdateDialog
          isOpen={showFirmwareUpdate}
          firmwareCheck={deviceStatus.firmwareCheck}
          onUpdateStart={() => {
            console.log('Starting firmware update for device:', selectedDeviceId)
            setShowFirmwareUpdate(false)
          }}
          onSkip={() => setShowFirmwareUpdate(false)}
          onRemindLater={() => setShowFirmwareUpdate(false)}
          onClose={() => setShowFirmwareUpdate(false)}
        />
      )}

      {verificationWizardOpen && !isOpen && (
        <SeedVerificationWizard
          isOpen={verificationWizardOpen}
          deviceId={verificationDeviceId || ''}
          deviceLabel={verificationDeviceLabel || ''}
          onClose={handleVerificationClose}
        />
      )}

    </>
  )
}

export const SettingsButton = ({ onClick }: { onClick: () => void }) => {
  return (
    <Button
      position="absolute"
      bottom="20px"
      left="20px"
      size="md"
      variant="ghost"
      colorScheme="blue"
      onClick={onClick}
      bg="rgba(0, 0, 0, 0.7)"
      _hover={{ bg: 'rgba(0, 0, 0, 0.8)' }}
    >
      <Icon as={FaCog} boxSize={5} />
    </Button>
  )
} 