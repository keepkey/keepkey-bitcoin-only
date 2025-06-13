import { useState, useEffect } from "react";
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import "./App.css";
import { Box, Text, Flex, Spinner } from "@chakra-ui/react";

import { Logo } from './components/logo/logo';
import splashBg from './assets/splash-bg.png'
import { EllipsisDots } from "./components/EllipsisSpinner";
import { SettingsDialog, SettingsButton } from './components/SettingsDialog';
import { useCommonDialogs } from './hooks/useCommonDialogs';
import { DeviceUpdateManager } from './components/DeviceUpdateManager';
import { useOnboardingState } from './hooks/useOnboardingState';
import { VaultInterface } from './components/VaultInterface';

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
    const { showOnboarding } = useCommonDialogs();
    const { shouldShowOnboarding, loading: onboardingLoading, clearCache } = useOnboardingState();
    
    // Function to restart backend startup process
    const handleLogoClick = async () => {
        if (isRestarting) return; // Prevent multiple clicks
        
        setIsRestarting(true);
        try {
            console.log("Logo clicked - restarting backend startup process");
            await invoke('restart_backend_startup');
            console.log("Backend restart initiated successfully");
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
                // Listen for the general application state events
                unlistenAppState = await listen('application:state', (event) => {
                    console.log('Application state event received:', event.payload);
                    const appState = event.payload as ApplicationState;
                    
                    // Update UI with status from backend
                    setLoadingStatus(appState.status);
                    setDeviceConnected(appState.connected);
                    
                    // Update device info if features are available
                    if (appState.features) {
                        setDeviceInfo({ 
                            features: appState.features, 
                            error: null 
                        });
                    } else if (!appState.connected) {
                        setDeviceInfo(null);
                    }
                });
            } catch (error) {
                console.error("Failed to set up event listeners:", error);
            }
        };

        setupEventListeners();

        return () => {
            if (unlistenAppState) unlistenAppState();
        };
    }, []); // Empty dependency array ensures this runs once on mount and cleans up on unmount

    // Show the main vault interface ONLY when device is ready (fully initialized and frontloaded)
    if (loadingStatus === "Device ready") {
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
            }}
          />


        </Flex>
      </Box>
    );
}

export default App;
