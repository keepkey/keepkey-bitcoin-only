import { useState, useEffect } from "react";
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import "./App.css";
import { Box, Text, Flex, Link, Stack, Button, Spinner } from "@chakra-ui/react";
import { FaCopy, FaCheck } from "react-icons/fa";

import { Logo } from './components/logo/logo';
import splashBg from './assets/splash-bg.png'
import { EllipsisDots } from "./components/EllipsisSpinner";
import { SettingsDialog, SettingsButton } from './components/SettingsDialog';
import { useCommonDialogs } from './hooks/useCommonDialogs';
import { DeviceUpdateManager } from './components/DeviceUpdateManager';
import { useOnboardingState } from './hooks/useOnboardingState';
import { VaultInterface } from './components/VaultInterface';
import { useWallet } from './contexts/WalletContext';

// Define the expected structure of DeviceFeatures from Rust
interface DeviceFeatures {
    label: string | null;
    vendor: string | null;
    model: string | null;
    firmware_variant: string | null;
    device_id: string | null;
    language: string | null;
    bootloader_mode: boolean;
    version: string;
    firmware_hash: string | null;
    bootloader_hash: string | null;
    initialized: boolean;
    imported: boolean | null;
    no_backup: boolean;
    pin_protection: boolean;
    pin_cached: boolean;
    passphrase_protection: boolean;
    passphrase_cached: boolean;
    wipe_code_protection: boolean;
    auto_lock_delay_ms: number | null;
    policies: string[];
}

interface DeviceInfoState {
    features: DeviceFeatures | null;
    error: string | null;
}

interface ApplicationState {
    status: string;
    connected: boolean;
    features: DeviceFeatures | null;
}

function App() {
    // We're tracking application state from backend events
    const [loadingStatus, setLoadingStatus] = useState<string>('Starting...');
    const [deviceConnected, setDeviceConnected] = useState<boolean>(false);
    const [, setDeviceInfo] = useState<DeviceInfoState | null>(null);
    const [isSettingsOpen, setIsSettingsOpen] = useState(false);
    const [isRestarting, setIsRestarting] = useState(false);
    const [deviceUpdateComplete, setDeviceUpdateComplete] = useState(false);
    const { showOnboarding, showError } = useCommonDialogs();
    const { shouldShowOnboarding, loading: onboardingLoading, clearCache } = useOnboardingState();
    
    // Function to show device access error dialog
    const showDeviceAccessError = (errorMessage: string) => {
        showError("KeepKey Device Access Error", errorMessage);
    };
    
    // Function to restart backend startup process
    const { reinitialize } = useWallet();

    const handleLogoClick = async () => {
        if (isRestarting) return; // Prevent multiple clicks
        
        setIsRestarting(true);
        try {
            console.log("Logo clicked - restarting backend startup process");
            await invoke('restart_backend_startup');
            console.log("Backend restart initiated successfully");
            // Re-run wallet initialization to resubscribe device state after backend restart
            reinitialize();
        } catch (error) {
            console.error("Failed to restart backend startup:", error);
        } finally {
            // Reset the restarting flag after a delay
            setTimeout(() => setIsRestarting(false), 2000);
        }
    };

    // Check onboarding status on startup
    useEffect(() => {
        if (onboardingLoading) {
            console.log("App.tsx: Onboarding state still loading...");
            return;
        }

        console.log(`App.tsx: Should show onboarding: ${shouldShowOnboarding}`);
        
        if (shouldShowOnboarding) {
            console.log("App.tsx: Showing onboarding wizard");
            // Add a small delay to ensure the dialog system is ready
            setTimeout(() => {
                showOnboarding({
                    onComplete: () => {
                        console.log("App.tsx: Onboarding completed callback");
                        clearCache(); // Clear the cache after completion
                    }
                });
            }, 1000);
        } else {
            console.log("App.tsx: Onboarding not needed, user is already onboarded");
        }
    }, [shouldShowOnboarding, onboardingLoading, showOnboarding, clearCache]);

    useEffect(() => {
        let unlistenAppState: (() => void) | undefined;

        const setupEventListeners = async () => {
            try {
                // Listen for device connected events (basic connection)
                const unlistenDeviceConnected = await listen('device:connected', (event) => {
                    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
                    const device = event.payload as any;
                    const deviceId: string = device.unique_id;
                    console.log('Device connected event received:', deviceId);
                    setLoadingStatus('Device detected – fetching features...');
                });

                // Listen for device ready events (device with features loaded)
                unlistenAppState = await listen('device:ready', (event) => {
                    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
                    const payload = event.payload as any;
                    console.log('Device ready event received:', payload);
                    
                    if (payload.device && payload.features) {
                        setDeviceConnected(true);
                        setDeviceInfo({ features: payload.features, error: null });
                        setLoadingStatus('Checking for updates...');
                        // Reset update completion state for new device connections
                        setDeviceUpdateComplete(false);
                        console.log(`✅ Device ready: ${payload.features.label || 'Unlabeled'} v${payload.features.version}`);
                    }
                });

                // Listen for device access errors from backend
                const unlistenAccessError = await listen('device:access-error', (event) => {
                    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
                    const errorData = event.payload as any;
                    console.log('Device access error event received:', errorData);
                    
                    showDeviceAccessError(errorData.error);
                    setLoadingStatus('Device in use by another app');
                });

                // Listen for device disconnection events
                const unlistenDeviceDisconnected = await listen('device:disconnected', (event) => {
                    console.log('Device disconnected event received:', event.payload);
                    setDeviceConnected(false);
                    setDeviceInfo(null);
                    setDeviceUpdateComplete(false);
                    setLoadingStatus('No device connected');
                });

                // Return cleanup function that removes all listeners
                return () => {
                    if (unlistenDeviceConnected) unlistenDeviceConnected();
                    if (unlistenAppState) unlistenAppState();
                    if (unlistenAccessError) unlistenAccessError();
                    if (unlistenDeviceDisconnected) unlistenDeviceDisconnected();
                };
                
            } catch (error) {
                console.error("Failed to set up event listeners:", error);
            }
        };

        setupEventListeners();

        return () => {
            if (unlistenAppState) unlistenAppState();
        };
    }, []); // Empty dependency array ensures this runs once on mount and cleans up on unmount

    const mcpUrl = "http://127.0.0.1:1646/mcp";
    const [hasCopied, setHasCopied] = useState(false);
    
    const handleCopy = () => {
      navigator.clipboard.writeText(mcpUrl);
      setHasCopied(true);
      setTimeout(() => setHasCopied(false), 2000);
    };

    // Show the main vault interface ONLY when device is ready AND updates are complete
    if (loadingStatus === "Device ready" && deviceConnected && deviceUpdateComplete) {
        return <VaultInterface />;
    }

    // Show splash screen while connecting
    return (
      <Box
        height="100vh"
        width="100vw"
        position="relative"
        backgroundImage={`url(${splashBg})`}
        backgroundSize="cover"
        backgroundPosition="center"
      >
        <Flex 
          height="100%"
          width="100%"
          direction="column"
          alignItems="center"
          justifyContent="center"
        >
          {/* Clickable Logo in the center */}
          <Logo 
            width="100px" 
            onClick={handleLogoClick}
            style={{
              filter: isRestarting ? 'brightness(1.3)' : 'none',
              transition: 'filter 0.2s ease'
            }}
          />
          
          {/* Clickable hint */}
          <Text 
            fontSize="xs" 
            color="gray.400" 
            mt={2} 
            textAlign="center"
            opacity={isRestarting ? 0.5 : 0.7}
            transition="opacity 0.2s ease"
          >
            {isRestarting ? "Restarting..." : ""}
          </Text>
          
          {/* Loading text at the bottom */}
            <Box
                position="absolute"
                bottom="40px"
                textAlign="center"
                width="auto"
                px={3}
                py={1}
                borderRadius="md"
                bg="rgba(0, 0, 0, 0.5)"
            >
                <Flex gap="2" justifyContent="center" alignItems="center">
                    <Spinner size="xs" color={deviceConnected ? "green.400" : "gray.400"} />
                    <Text fontSize="xs" color="gray.300">
                        {loadingStatus}
                    </Text>
                    <EllipsisDots interval={300} /> {/* ⟵ no layout shift */}
                </Flex>
            </Box>

          {/* Settings button in bottom left */}
          <SettingsButton onClick={() => setIsSettingsOpen(true)} />

          {/* Settings dialog */}
          <SettingsDialog 
            isOpen={isSettingsOpen} 
            onClose={() => setIsSettingsOpen(false)} 
          />

          {/* Device update manager - handles bootloader/firmware updates and wallet creation */}
          <DeviceUpdateManager 
            onComplete={() => {
              console.log('Device update/initialization complete');
              setDeviceUpdateComplete(true);
              setLoadingStatus('Device ready');
            }}
          />

          {/* REST and MCP links in bottom right corner */}
          {/*<Box*/}
          {/*  position="absolute"*/}
          {/*  bottom="20px"*/}
          {/*  right="20px"*/}
          {/*  background="rgba(0, 0, 0, 0.7)"*/}
          {/*  borderRadius="md"*/}
          {/*  boxShadow="md"*/}
          {/*  padding={2}*/}
          {/*  border="1px solid rgba(100, 255, 100, 0.3)"*/}
          {/*>*/}
          {/*  <Flex direction="column" gap={1}>*/}
          {/*    <Stack direction="row" gap={2} align="center">*/}
          {/*      <Text fontSize="2xs" color="gray.300">REST:</Text>*/}
          {/*      <Link */}
          {/*        href="http://127.0.0.1:1646/docs" */}
          {/*        target="_blank" */}
          {/*        fontSize="xs" */}
          {/*        color="blue.300"*/}
          {/*        _hover={{ color: "blue.200", textDecoration: "underline" }}*/}
          {/*      >*/}
          {/*        http://127.0.0.1:1646/docs*/}
          {/*      </Link>*/}
          {/*    </Stack>*/}
          {/*    <Stack direction="row" gap={2} align="center">*/}
          {/*      <Text fontSize="2xs" color="gray.300">MCP:</Text>*/}
          {/*      <Link */}
          {/*        href="http://127.0.0.1:1646/mcp" */}
          {/*        target="_blank" */}
          {/*        fontSize="xs" */}
          {/*        color="blue.300"*/}
          {/*        _hover={{ color: "blue.200", textDecoration: "underline" }}*/}
          {/*      >*/}
          {/*        http://127.0.0.1:1646/mcp*/}
          {/*      </Link>*/}
          {/*      <Box position="relative">*/}
          {/*        <Button*/}
          {/*          size="xs"*/}
          {/*          variant="ghost"*/}
          {/*          colorScheme={hasCopied ? "green" : "blue"}*/}
          {/*          aria-label="Copy MCP URL to clipboard"*/}
          {/*          title={hasCopied ? "Copied!" : "Copy to clipboard"}*/}
          {/*          onClick={handleCopy}*/}
          {/*          p={1}*/}
          {/*          minW={0}*/}
          {/*          height="16px"*/}
          {/*        >*/}
          {/*          {hasCopied ? <FaCheck size="10px" /> : <FaCopy size="10px" />}*/}
          {/*        </Button>*/}
          {/*      </Box>*/}
          {/*    </Stack>*/}
          {/*  </Flex>*/}
          {/*</Box>*/}
        </Flex>
      </Box>
    );
}

export default App;
