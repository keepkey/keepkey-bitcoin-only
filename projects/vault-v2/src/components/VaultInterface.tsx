import { useState, useEffect, useCallback } from 'react';
import { Box, Flex, Button, Text, HStack, useDisclosure } from '@chakra-ui/react';
import { FaTh, FaGlobe, FaWallet, FaCog, FaQuestionCircle } from 'react-icons/fa';
import { listen, emit } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import splashBg from '../assets/splash-bg.png';
import { SettingsDialog } from './SettingsDialog';
import { AppsView, BrowserView, PairingsView, VaultView, AssetView } from './views';
import { WalletProvider, useWallet } from '../contexts/WalletContext';
import Send from './Send';
import Receive from './Receive';
import { useDialog } from '../contexts/DialogContext';
import packageJson from '../../package.json';
// import { AppHeader } from './AppHeader';

type ViewType = 'apps' | 'browser' | 'pairings' | 'vault' | 'assets' | 'send' | 'receive' | 'portfolio';

interface NavItem {
  id: ViewType | 'settings' | 'support';
  label: string;
  icon: React.ReactNode;
  onClick: () => void;
}

export const VaultInterface = () => {
  const [currentView, setCurrentView] = useState<ViewType>('portfolio');
  const { open: isSettingsOpen, onOpen: openSettings, onClose: closeSettings } = useDisclosure();
  const [isRecoveryWizardOpen, setIsRecoveryWizardOpen] = useState(false);
  const { refreshPortfolio } = useWallet();
  const { hideAll } = useDialog();

  // Clear any stuck dialogs when component mounts
  useEffect(() => {
    console.log('🏦 VaultInterface mounted - clearing any stuck dialogs');
    hideAll();
  }, [hideAll]);

  const handleViewChange = async (view: ViewType) => {
    try {
      await invoke('vault_change_view', { view });
    } catch (error) {
      console.error('Failed to notify backend of view change:', error);
      // Fallback to direct change if backend fails
      setCurrentView(view);
    }
  };

  const handleSupportClick = async () => {
    try {
      // First try to open in integrated browser
      await invoke('vault_open_support');
      console.log('Opening support in integrated browser');
    } catch (error) {
      console.error('Failed to open support via integrated browser:', error);
      // Fallback to opening in external browser
      try {
        await invoke('open_url', { url: 'https://support.keepkey.com' });
        console.log('Opening support in external browser');
      } catch (fallbackError) {
        console.error('Failed to open URL via Tauri:', fallbackError);
        // Last resort: use window.open
        window.open('https://support.keepkey.com', '_blank');
        console.log('Opening support via window.open');
      }
    }
  };

  // Function to navigate to send/receive from portfolio
  const navigateToSendReceive = (action: 'send' | 'receive') => {
    setCurrentView(action);
  };

  const navItems: NavItem[] = [
    {
      id: 'vault',
      label: 'Vault',
      icon: <FaWallet />,
      onClick: () => handleViewChange('vault'),
    },
    // {
    //   id: 'browser',
    //   label: 'Browser',
    //   icon: <FaGlobe />,
    //   onClick: async () => {
    //     handleViewChange('browser');
    //     // Always navigate to keepkey.com when browser button is clicked
    //     setTimeout(async () => {
    //       await emit('browser:navigate', { url: 'http://localhost:8080' });
    //     }, 100);
    //   },
    // },
    {
      id: 'settings',
      label: 'Settings',
      icon: <FaCog />,
      onClick: openSettings,
    },
    {
      id: 'support',
      label: 'Support',
      icon: <FaQuestionCircle />,
      onClick: handleSupportClick,
    },
  ];

  // Listen for backend view change commands
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupEventListeners = async () => {
      try {
        // Listen for backend view change commands
        unlisten = await listen('vault:change_view', (event) => {
          const { view } = event.payload as { view: ViewType };
          console.log('Received backend view change command:', view);
          setCurrentView(view);
        });
      } catch (error) {
        console.error('Failed to set up vault event listeners:', error);
      }
    };

    setupEventListeners();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const renderCurrentView = () => {
    switch (currentView) {
      case 'apps':
        return <AppsView />;
      case 'vault':
        return <VaultView onNavigate={navigateToSendReceive} />;
      case 'assets':
        return <AssetView />;
      case 'browser':
        return <BrowserView />;
      case 'pairings':
        return <PairingsView />;
      case 'send':
        return <Send onBack={() => setCurrentView('vault')} />;
      case 'receive':
        return <Receive onBack={() => setCurrentView('vault')} />;
      case 'portfolio':
        return <VaultView onNavigate={navigateToSendReceive} />;
      default:
        return <VaultView onNavigate={navigateToSendReceive} />;
    }
  };

  const handleRecoveryComplete = useCallback(async (success: boolean) => {
    // ... existing code ...
  }, []);

  return (
    <WalletProvider>
      <Box 
        height="100vh" 
        width="100vw" 
        position="relative"
        backgroundImage={`url(${splashBg})`}
        backgroundSize="cover"
        backgroundPosition="center"
      >
      {/* Main Vault Interface - Hidden when settings is open */}
      {!isSettingsOpen && (
        <Box height="100%" display="flex" flexDirection="column">
          {/* Top Navigation Bar */}
          <Box
            height="60px"
            bg="gray.900"
            borderBottom="1px solid"
            borderColor="gray.700"
            px={4}
            py={2}
          >
            <HStack justify="space-between" align="center" height="100%">
              {/* Left side - Main navigation items */}
              <HStack spacing={2}>
                {navItems.slice(0, 2).map((item) => (
                  <Button
                    key={item.id}
                    variant="ghost"
                    size="sm"
                    height="40px"
                    minWidth="80px"
                    gap={2}
                    color={
                      (item.id === currentView) 
                        ? "blue.400" 
                        : "gray.400"
                    }
                    _hover={{
                      color: "blue.300",
                      bg: "gray.800",
                    }}
                    _active={{
                      bg: "gray.700",
                    }}
                    onClick={item.onClick}
                  >
                    <Box fontSize="md">{item.icon}</Box>
                    <Text fontSize="sm" fontWeight="medium">
                      {item.label}
                    </Text>
                  </Button>
                ))}
              </HStack>

              {/* Center - Logo/Title */}
              <Text fontSize="lg" fontWeight="bold" color="white">
                KeepKey Vault
              </Text>

              {/* Right side - Settings and Support */}
              <HStack spacing={2}>
                {navItems.slice(2).map((item) => (
                  <Button
                    key={item.id}
                    variant="ghost"
                    size="sm"
                    height="40px"
                    minWidth="80px"
                    gap={2}
                    color="gray.400"
                    _hover={{
                      color: "blue.300",
                      bg: "gray.800",
                    }}
                    _active={{
                      bg: "gray.700",
                    }}
                    onClick={item.onClick}
                  >
                    <Box fontSize="md">{item.icon}</Box>
                    <Text fontSize="sm" fontWeight="medium">
                      {item.label}
                    </Text>
                  </Button>
                ))}
              </HStack>
            </HStack>
          </Box>
          
          {/* Main Content Area */}
          <Box flex="1" overflow="hidden">
            {renderCurrentView()}
          </Box>
        </Box>
      )}

      {/* Full-Screen Settings Overlay */}
      {isSettingsOpen && (
        <Box
          position="absolute"
          top="0"
          left="0"
          right="0"
          bottom="0"
          backgroundImage={`url(${splashBg})`}
          backgroundSize="cover"
          backgroundPosition="center"
          zIndex="modal"
          display="flex"
          alignItems="center"
          justifyContent="center"
        >
          <SettingsDialog isOpen={isSettingsOpen} onClose={closeSettings} />
        </Box>
      )}
      </Box>
    </WalletProvider>
  );
}; 