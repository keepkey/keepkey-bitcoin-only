import { Tabs, VStack, Text, Button, Icon, Box, HStack, Flex, Link, Stack, Input, Textarea, Spinner } from '@chakra-ui/react'
import { 
  DialogRoot,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogBody,
  DialogCloseTrigger
} from './ui/dialog'
import { LuSettings, LuMonitor, LuCpu, LuNetwork, LuFileText } from 'react-icons/lu'
import { FaCog, FaLink, FaCopy, FaCheck, FaTimes, FaUsb, FaLock, FaGlobe, FaDollarSign, FaDownload, FaTrash, FaSyncAlt, FaSearch, FaFilter } from 'react-icons/fa'
import { useState, useEffect } from 'react'

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

interface LogEntry {
  timestamp: string
  direction: string
  device_id?: string
  request_id?: string
  request_type?: string
  message_type?: string
  success?: boolean
  data: any
  error?: string
}

export const SettingsDialog = ({ isOpen, onClose }: SettingsDialogProps) => {
  const [selectedDeviceId, setSelectedDeviceId] = useState<string | null>(null)
  const [showBootloaderUpdate, setShowBootloaderUpdate] = useState(false)
  const [showFirmwareUpdate, setShowFirmwareUpdate] = useState(false)
  const [deviceStatus, setDeviceStatus] = useState<DeviceStatus | null>(null)
  
  // Copy state for URLs
  const [hasCopiedMcp, setHasCopiedMcp] = useState(false)
  const [hasCopiedRest, setHasCopiedRest] = useState(false)
  
  // Seed verification wizard state
  const [verificationWizardOpen, setVerificationWizardOpen] = useState(false)
  const [verificationDeviceId, setVerificationDeviceId] = useState<string | null>(null)
  const [verificationDeviceLabel, setVerificationDeviceLabel] = useState<string | null>(null)
  
  // Log viewer state
  const [logs, setLogs] = useState<LogEntry[]>([])
  const [isLoadingLogs, setIsLoadingLogs] = useState(false)
  const [logFilter, setLogFilter] = useState<string>('all') // 'all', 'requests', 'responses', 'errors'
  const [searchTerm, setSearchTerm] = useState<string>('')
  const [logLimit, setLogLimit] = useState<number>(50)
  const [logPath, setLogPath] = useState<string>('')
  const [autoRefresh, setAutoRefresh] = useState<boolean>(false)
  const [refreshInterval, setRefreshInterval] = useState<NodeJS.Timeout | null>(null)
  const [lastRefresh, setLastRefresh] = useState<Date | null>(null)
  
  const firmwareWizard = useFirmwareUpdateWizard()
  const walletCreationWizard = useWalletCreationWizard()
  
  // URL copy handlers
  const handleCopyMcp = () => {
    navigator.clipboard.writeText("http://127.0.0.1:1646/mcp");
    setHasCopiedMcp(true);
    setTimeout(() => setHasCopiedMcp(false), 2000);
  };

  const handleCopyRest = () => {
    navigator.clipboard.writeText("http://127.0.0.1:1646/docs");
    setHasCopiedRest(true);
    setTimeout(() => setHasCopiedRest(false), 2000);
  };
  
  // Define fetchDeviceStatus function outside of useEffect
  const fetchDeviceStatus = async (deviceId = selectedDeviceId) => {
  console.debug('[SettingsDialog] fetchDeviceStatus called with deviceId:', deviceId);
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

  const handleOpenPairings = async () => {
    try {
      // First close the settings dialog
      onClose()
      
      // Then notify backend to open pairings view
      await invoke('vault_change_view', { view: 'pairings' })
    } catch (error) {
      console.error('Failed to open pairings view:', error)
      // Fallback - could emit an event or handle differently
    }
  }

  // Log viewer functions
  const loadLogs = async () => {
    setIsLoadingLogs(true)
    try {
      const recentLogs = await invoke<LogEntry[]>('get_recent_device_logs', { limit: logLimit })
      setLogs(recentLogs)
      
      // Also get the log file path
      const path = await invoke<string>('get_device_log_path')
      setLogPath(path)
      
      // Update last refresh time
      setLastRefresh(new Date())
    } catch (error) {
      console.error('Failed to load logs:', error)
      setLogs([])
    } finally {
      setIsLoadingLogs(false)
    }
  }

  const handleDownloadLogs = async () => {
    try {
      // Get the log file path
      const path = await invoke<string>('get_device_log_path')
      
      // Copy the path to clipboard and notify user where to find the logs
      await navigator.clipboard.writeText(path)
      
      // Show detailed information about where logs are stored
      const fileName = path.split('/').pop() || 'device-communications.log'
      alert(`Log file location copied to clipboard!\n\nFile: ${fileName}\nLocation: ${path}\n\nYou can navigate to this location to access your logs.`)
    } catch (error) {
      console.error('Failed to get log path:', error)
      alert('Failed to get log file path')
    }
  }

  const handleCleanupLogs = async () => {
    try {
      await invoke('cleanup_device_logs')
      alert('Old logs cleaned up successfully')
      // Reload logs after cleanup
      await loadLogs()
    } catch (error) {
      console.error('Failed to cleanup logs:', error)
      alert('Failed to cleanup old logs')
    }
  }

  const filteredLogs = logs.filter(log => {
    // Apply filter
    if (logFilter === 'requests' && log.direction !== 'REQUEST' && log.direction !== 'SEND') return false
    if (logFilter === 'responses' && log.direction !== 'RESPONSE' && log.direction !== 'RECEIVE') return false
    if (logFilter === 'errors' && log.success !== false && !log.error) return false
    
    // Apply search to the formatted terminal output
    if (searchTerm) {
      const searchLower = searchTerm.toLowerCase()
      const formattedOutput = formatLogEntry(log).toLowerCase()
      const deviceId = (log.device_id || '').toLowerCase()
      const requestType = (log.request_type || log.message_type || '').toLowerCase()
      
      return formattedOutput.includes(searchLower) || 
             deviceId.includes(searchLower) || 
             requestType.includes(searchLower)
    }
    
    return true
  })

  const formatLogEntry = (log: LogEntry) => {
    const timestamp = new Date(log.timestamp).toLocaleString()
    
    // Format logs to look like terminal output
    if (log.direction === 'REQUEST' || log.direction === 'SEND') {
      // Outgoing requests
      const requestType = log.request_type || log.message_type || 'Unknown'
      return `-> ${requestType}`
    } else if (log.direction === 'RESPONSE' || log.direction === 'RECEIVE') {
      // Incoming responses
      const requestType = log.request_type || log.message_type || 'Unknown'
      const success = log.success !== false ? '✅' : '❌'
      
      // Try to extract meaningful info from the response
      let responseInfo = ''
      if (log.data) {
        // Handle GetFeatures responses
        if (log.data.features && log.data.features.label) {
          responseInfo = `Features: ${log.data.features.label} v${log.data.features.version || 'Unknown'}`
        } else if (log.data.status && log.data.status.features && log.data.status.features.label) {
          responseInfo = `Status: ${log.data.status.features.label} v${log.data.status.features.version || 'Unknown'}`
        } else if (log.data.response && typeof log.data.response === 'string') {
          // For xpub responses, show truncated version
          if (log.data.response.startsWith('xpub') || log.data.response.startsWith('ypub') || log.data.response.startsWith('zpub')) {
            responseInfo = `${log.data.response.substring(0, 16)}...`
          } else {
            responseInfo = log.data.response
          }
        } else if (requestType === 'GetFeatures' && log.data.operation) {
          responseInfo = 'Features received'
        } else {
          responseInfo = requestType.replace('Get', '').replace('Request', '')
        }
      }
      
      return `<- ${responseInfo} ${success}`
    } else {
      // Other log types (like status messages)
      const deviceShort = log.device_id ? log.device_id.substring(log.device_id.length - 8) : 'system'
      
      // Handle different operation types
      if (log.data && log.data.operation) {
        const operation = log.data.operation
        if (operation === 'get_device_status') {
          return `Getting device status for: ${log.device_id}`
        } else if (operation === 'get_features_for_device') {
          return `📡 Fetching device features for: ${deviceShort}`
        } else if (log.data.status) {
          const status = log.data.status
          if (status.bootloaderCheck) {
            return `🔧 Bootloader check: ${status.bootloaderCheck.currentVersion} -> needs update: ${status.bootloaderCheck.needsUpdate} (bootloader_mode: ${status.features?.bootloader_mode || false})`
          } else if (status.firmwareCheck) {
            return `🔧 Firmware check: ${status.firmwareCheck.currentVersion} vs ${status.firmwareCheck.latestVersion} -> needs update: ${status.firmwareCheck.needsUpdate} (bootloader_mode: ${status.features?.bootloader_mode || false})`
          } else if (status.initializationCheck) {
            return `🔧 Initialization check: initialized=${status.initializationCheck.initialized}, needs_setup=${status.initializationCheck.needsSetup}, has_pin_protection=${status.features?.pin_protection || false}, pin_cached=${status.features?.pin_cached || false}`
          }
        }
        return `${operation.replace(/_/g, ' ')}: ${deviceShort}`
      }
      
      return `${log.direction}: ${JSON.stringify(log.data).substring(0, 100)}...`
    }
  }

  // Load logs when the dialog opens
  useEffect(() => {
    if (isOpen) {
      loadLogs()
    }
  }, [isOpen, logLimit])

  // Auto-refresh functionality
  useEffect(() => {
    if (autoRefresh && isOpen) {
      const interval = setInterval(() => {
        loadLogs()
      }, 2000) // Refresh every 2 seconds
      setRefreshInterval(interval)
      
      return () => {
        clearInterval(interval)
      }
    } else {
      if (refreshInterval) {
        clearInterval(refreshInterval)
        setRefreshInterval(null)
      }
    }
  }, [autoRefresh, isOpen, logLimit])

  // Auto-scroll to bottom when new logs arrive (if auto-refresh is on)
  useEffect(() => {
    if (autoRefresh && logs.length > 0) {
      const logContainer = document.getElementById('log-container')
      if (logContainer) {
        logContainer.scrollTop = logContainer.scrollHeight
      }
    }
  }, [logs, autoRefresh])
  
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
            <DialogCloseTrigger color="gray.400" _hover={{ color: "white" }}>
              <FaTimes />
            </DialogCloseTrigger>
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
                  value="logs"
                  flex="1"
                  gap={2}
                  color="gray.400"
                  _selected={{ bg: "gray.700", color: "white" }}
                  _hover={{ color: "white" }}
                >
                  <LuFileText size={16} />
                  Logs
                </Tabs.Trigger>
                <Tabs.Trigger 
                  value="mcp"
                  flex="1"
                  gap={2}
                  color="gray.600"
                  opacity={0.5}
                  cursor="not-allowed"
                  _selected={{ bg: "gray.800", color: "gray.500" }}
                  _hover={{ color: "gray.600" }}
                  disabled
                >
                  <FaLock size={14} />
                  MCP
                </Tabs.Trigger>
              </Tabs.List>

              <Tabs.Content value="general" minHeight="400px" overflowY="auto">
                <VStack align="stretch" gap={4}>
                  <Text color="white" fontSize="lg" fontWeight="semibold">General Settings</Text>
                  
                  {/* Language Settings */}
                  <Box bg="gray.800" p={4} borderRadius="md" border="1px solid" borderColor="gray.700" opacity={0.6}>
                    <VStack align="stretch" gap={3}>
                      <HStack gap={2}>
                        <FaGlobe color="gray.500" />
                        <Text color="gray.400" fontWeight="medium">Language Settings</Text>
                        <FaLock color="gray.500" size={12} />
                      </HStack>
                      <HStack justify="space-between" align="center">
                        <Text color="gray.400" fontSize="sm">Display Language</Text>
                        <Text color="gray.500" fontSize="sm">English (Locked)</Text>
                      </HStack>
                      <Text color="gray.500" fontSize="xs">
                        Language selection will be available in a future update
                      </Text>
                    </VStack>
                  </Box>

                  {/* Currency Settings */}
                  <Box bg="gray.800" p={4} borderRadius="md" border="1px solid" borderColor="gray.700" opacity={0.6}>
                    <VStack align="stretch" gap={3}>
                      <HStack gap={2}>
                        <FaDollarSign color="gray.500" />
                        <Text color="gray.400" fontWeight="medium">Currency & Format Settings</Text>
                        <FaLock color="gray.500" size={12} />
                      </HStack>
                      <HStack justify="space-between" align="center">
                        <Text color="gray.400" fontSize="sm">Primary Currency</Text>
                        <Text color="gray.500" fontSize="sm">USD (Locked)</Text>
                      </HStack>
                      <HStack justify="space-between" align="center">
                        <Text color="gray.400" fontSize="sm">Number Format</Text>
                        <Text color="gray.500" fontSize="sm">1,000.00 (Locked)</Text>
                      </HStack>
                      <Text color="gray.500" fontSize="xs">
                        Currency and formatting options will be available in a future update
                      </Text>
                    </VStack>
                  </Box>
                </VStack>
              </Tabs.Content>

              <Tabs.Content value="app" minHeight="400px" overflowY="auto">
                <VStack align="stretch" gap={4}>
                  <Text color="white" fontSize="lg" fontWeight="semibold">Application Settings</Text>
                  
                  {/* Pairings Section */}
                  <Box bg="gray.800" p={4} borderRadius="md" border="1px solid" borderColor="gray.700">
                    <VStack align="stretch" gap={3}>
                      <HStack justify="space-between" align="center">
                        <VStack align="start" gap={1}>
                          <Text color="white" fontWeight="medium">Device Pairings</Text>
                          <Text color="gray.400" fontSize="sm">
                            Manage connected applications and services
                          </Text>
                        </VStack>
                        <Button
                          colorScheme="blue"
                          size="sm"
                          onClick={handleOpenPairings}
                        >
                          <HStack gap={2}>
                            <FaLink />
                            <Text>Open Pairings</Text>
                          </HStack>
                        </Button>
                      </HStack>
                    </VStack>
                  </Box>

                  {/* Future app settings can go here */}
                  <Text color="gray.400" fontSize="sm">More app settings coming soon...</Text>
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

              <Tabs.Content value="logs" minHeight="400px" overflowY="auto">
                <VStack align="stretch" gap={4}>
                                      <HStack justify="space-between" align="center">
                      <Text color="white" fontSize="lg" fontWeight="semibold">Device Communication Logs</Text>
                      <HStack gap={2}>
                        <Button
                          size="sm"
                          colorScheme={autoRefresh ? "orange" : "blue"}
                          onClick={() => setAutoRefresh(!autoRefresh)}
                        >
                          <HStack gap={1}>
                            <FaSyncAlt />
                            <Text>{autoRefresh ? 'Stop Auto' : 'Auto Refresh'}</Text>
                          </HStack>
                        </Button>
                        <Button
                          size="sm"
                          colorScheme="blue"
                          onClick={loadLogs}
                          loading={isLoadingLogs}
                          disabled={autoRefresh}
                        >
                          <HStack gap={1}>
                            <FaSyncAlt />
                            <Text>{isLoadingLogs ? 'Loading...' : 'Refresh'}</Text>
                          </HStack>
                        </Button>
                        <Button
                          size="sm"
                          colorScheme="green"
                          onClick={handleDownloadLogs}
                        >
                          <HStack gap={1}>
                            <FaDownload />
                            <Text>Download</Text>
                          </HStack>
                        </Button>
                        <Button
                          size="sm"
                          colorScheme="red"
                          onClick={handleCleanupLogs}
                        >
                          <HStack gap={1}>
                            <FaTrash />
                            <Text>Cleanup</Text>
                          </HStack>
                        </Button>
                      </HStack>
                    </HStack>

                  {/* Log Controls */}
                  <Box bg="gray.800" p={4} borderRadius="md" border="1px solid" borderColor="gray.700">
                    <VStack align="stretch" gap={3}>
                      <HStack gap={4} wrap="wrap">
                        <HStack flex="1" minW="200px">
                          <FaSearch color="gray.400" />
                          <Input
                            placeholder="Search logs..."
                            value={searchTerm}
                            onChange={(e) => setSearchTerm(e.target.value)}
                            bg="gray.900"
                            border="1px solid"
                            borderColor="gray.600"
                            color="white"
                            _placeholder={{ color: "gray.400" }}
                            size="sm"
                          />
                        </HStack>
                        <HStack>
                          <FaFilter color="gray.400" />
                          <select
                            value={logFilter}
                            onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setLogFilter(e.target.value)}
                            style={{
                              backgroundColor: 'var(--chakra-colors-gray-900)',
                              border: '1px solid var(--chakra-colors-gray-600)',
                              color: 'white',
                              padding: '4px 8px',
                              borderRadius: '4px',
                              minWidth: '120px'
                            }}
                          >
                            <option value="all">All</option>
                            <option value="requests">Requests</option>
                            <option value="responses">Responses</option>
                            <option value="errors">Errors</option>
                          </select>
                        </HStack>
                        <HStack>
                          <Text color="gray.400" fontSize="sm">Limit:</Text>
                          <select
                            value={logLimit.toString()}
                            onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setLogLimit(parseInt(e.target.value))}
                            style={{
                              backgroundColor: 'var(--chakra-colors-gray-900)',
                              border: '1px solid var(--chakra-colors-gray-600)',
                              color: 'white',
                              padding: '4px 8px',
                              borderRadius: '4px',
                              minWidth: '80px'
                            }}
                          >
                            <option value="25">25</option>
                            <option value="50">50</option>
                            <option value="100">100</option>
                            <option value="200">200</option>
                          </select>
                        </HStack>
                      </HStack>
                      
                      {logPath && (
                        <HStack gap={2}>
                          <Text color="gray.400" fontSize="xs">Log file:</Text>
                          <Text color="blue.300" fontSize="xs" fontFamily="mono" wordBreak="break-all">
                            {logPath}
                          </Text>
                        </HStack>
                      )}
                    </VStack>
                  </Box>

                                    {/* Terminal-style Log Display */}
                  <Box 
                    id="log-container"
                    bg="black" 
                    borderRadius="md" 
                    border="1px solid" 
                    borderColor="gray.600" 
                    maxH="400px" 
                    overflowY="auto"
                    p={4}
                    position="relative"
                  >
                    {autoRefresh && (
                      <Box
                        position="absolute"
                        top={2}
                        right={2}
                        bg="orange.600"
                        color="white"
                        px={2}
                        py={1}
                        fontSize="xs"
                        borderRadius="sm"
                        fontWeight="bold"
                        zIndex={1}
                      >
                        LIVE
                      </Box>
                    )}
                    {isLoadingLogs ? (
                      <Flex justify="center" align="center" p={8}>
                        <VStack gap={2}>
                          <Spinner color="green.400" />
                          <Text color="green.400" fontFamily="mono">Loading logs...</Text>
                        </VStack>
                      </Flex>
                    ) : filteredLogs.length === 0 ? (
                      <Flex justify="center" align="center" p={8}>
                        <Text color="green.400" fontFamily="mono">No logs found matching current filters</Text>
                      </Flex>
                    ) : (
                      <VStack align="stretch" gap={1}>
                        {filteredLogs.map((log, index) => {
                          const formatted = formatLogEntry(log)
                          const timestamp = new Date(log.timestamp).toLocaleString()
                          
                          // Color coding based on log type
                          let color = "green.300"
                          if (formatted.startsWith('->')) {
                            color = "blue.300" // Outgoing requests
                          } else if (formatted.startsWith('<-')) {
                            color = formatted.includes('❌') ? "red.300" : "green.300" // Responses
                          } else if (formatted.includes('🔧')) {
                            color = "yellow.300" // System checks
                          } else if (formatted.includes('📡')) {
                            color = "cyan.300" // Network/communication
                          }
                          
                          return (
                            <HStack key={index} gap={4} align="start" fontSize="sm" fontFamily="mono">
                              <Text color="gray.500" fontSize="xs" minW="140px" flexShrink={0}>
                                {timestamp}
                              </Text>
                              <Text color={color} wordBreak="break-word">
                                {formatted}
                              </Text>
                            </HStack>
                          )
                        })}
                      </VStack>
                    )}
                  </Box>

                  {/* Log Stats */}
                  <HStack justify="space-between" align="center" color="gray.400" fontSize="sm" wrap="wrap">
                    <Text>Showing {filteredLogs.length} of {logs.length} log entries</Text>
                    <HStack gap={4}>
                      {lastRefresh && (
                        <Text>Last updated: {lastRefresh.toLocaleTimeString()}</Text>
                      )}
                      {autoRefresh && (
                        <Text color="orange.400" fontWeight="bold">Auto-refreshing every 2s</Text>
                      )}
                    </HStack>
                  </HStack>
                </VStack>
              </Tabs.Content>

              <Tabs.Content value="mcp" minHeight="400px" overflowY="auto">
                <VStack align="stretch" gap={4}>
                  <HStack gap={2}>
                    <FaLock color="gray.500" />
                    <Text color="gray.500" fontSize="lg" fontWeight="semibold">MCP & API Access</Text>
                    <Text color="red.400" fontSize="sm" fontWeight="medium">(Disabled)</Text>
                  </HStack>
                  
                  {/* API URLs Section */}
                  <Box bg="gray.800" p={4} borderRadius="md" border="1px solid" borderColor="gray.700" opacity={0.5}>
                    <VStack align="stretch" gap={4}>
                      <HStack gap={2}>
                        <FaLock color="gray.500" size={14} />
                        <Text color="gray.400" fontWeight="medium">API Endpoints</Text>
                        <Text color="red.400" fontSize="xs">(Disabled)</Text>
                      </HStack>
                      
                      {/* REST API */}
                      <Box bg="gray.900" p={3} borderRadius="md">
                        <VStack align="stretch" gap={2}>
                          <Text color="gray.300" fontSize="sm" fontWeight="medium">REST API Documentation</Text>
                          <HStack justify="space-between" align="center">
                            <Link 
                              href="http://127.0.0.1:1646/docs" 
                              target="_blank" 
                              fontSize="sm" 
                              color="blue.300"
                              _hover={{ color: "blue.200", textDecoration: "underline" }}
                              flex="1"
                            >
                              http://127.0.0.1:1646/docs
                            </Link>
                            <Button
                              size="xs"
                              variant="ghost"
                              colorScheme={hasCopiedRest ? "green" : "blue"}
                              aria-label="Copy REST URL to clipboard"
                              onClick={handleCopyRest}
                              minW="60px"
                            >
                              {hasCopiedRest ? (
                                <HStack gap={1}>
                                  <FaCheck size="10px" />
                                  <Text fontSize="xs">Copied</Text>
                                </HStack>
                              ) : (
                                <HStack gap={1}>
                                  <FaCopy size="10px" />
                                  <Text fontSize="xs">Copy</Text>
                                </HStack>
                              )}
                            </Button>
                          </HStack>
                          <Text color="gray.500" fontSize="xs">
                            Interactive API documentation and testing interface
                          </Text>
                        </VStack>
                      </Box>

                      {/* MCP Endpoint */}
                      <Box bg="gray.900" p={3} borderRadius="md">
                        <VStack align="stretch" gap={2}>
                          <Text color="gray.300" fontSize="sm" fontWeight="medium">MCP (Model Context Protocol)</Text>
                          <HStack justify="space-between" align="center">
                            <Link 
                              href="http://127.0.0.1:1646/mcp" 
                              target="_blank" 
                              fontSize="sm" 
                              color="blue.300"
                              _hover={{ color: "blue.200", textDecoration: "underline" }}
                              flex="1"
                            >
                              http://127.0.0.1:1646/mcp
                            </Link>
                            <Button
                              size="xs"
                              variant="ghost"
                              colorScheme={hasCopiedMcp ? "green" : "blue"}
                              aria-label="Copy MCP URL to clipboard"
                              onClick={handleCopyMcp}
                              minW="60px"
                            >
                              {hasCopiedMcp ? (
                                <HStack gap={1}>
                                  <FaCheck size="10px" />
                                  <Text fontSize="xs">Copied</Text>
                                </HStack>
                              ) : (
                                <HStack gap={1}>
                                  <FaCopy size="10px" />
                                  <Text fontSize="xs">Copy</Text>
                                </HStack>
                              )}
                            </Button>
                          </HStack>
                          <Text color="gray.500" fontSize="xs">
                            AI assistant integration endpoint for Claude and other LLMs
                          </Text>
                        </VStack>
                      </Box>
                    </VStack>
                  </Box>

                  {/* Status Section */}
                  <Box bg="gray.800" p={4} borderRadius="md" border="1px solid" borderColor="gray.700" opacity={0.6}>
                    <VStack align="stretch" gap={3}>
                      <HStack gap={2}>
                        <FaLock color="gray.500" size={14} />
                        <Text color="gray.400" fontWeight="medium">Service Status</Text>
                      </HStack>
                      <HStack justify="space-between" align="center">
                        <Text color="gray.400" fontSize="sm">Backend Server</Text>
                        <HStack gap={2}>
                          <Box w={2} h={2} bg="gray.500" borderRadius="full" />
                          <Text color="gray.500" fontSize="sm" fontWeight="medium">Offline</Text>
                        </HStack>
                      </HStack>
                      <HStack justify="space-between" align="center">
                        <Text color="gray.400" fontSize="sm">API Access</Text>
                        <Text color="gray.500" fontSize="sm" fontFamily="mono">Locked</Text>
                      </HStack>
                    </VStack>
                  </Box>

                  {/* Information Section */}
                  <Box bg="gray.800" p={4} borderRadius="md" border="1px solid" borderColor="gray.700" opacity={0.4}>
                    <VStack align="stretch" gap={2}>
                      <HStack gap={2}>
                        <FaLock color="gray.500" size={12} />
                        <Text color="gray.400" fontSize="sm" fontWeight="medium">About MCP</Text>
                      </HStack>
                      <Text color="gray.500" fontSize="xs">
                        The Model Context Protocol (MCP) would enable AI assistants like Claude to securely interact with your KeepKey device. 
                        This feature is currently disabled and will be available in a future update.
                      </Text>
                    </VStack>
                  </Box>
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