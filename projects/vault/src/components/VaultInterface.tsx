import { useState } from 'react';
import { Box, Flex, Button, Text, HStack, useDisclosure } from '@chakra-ui/react';
import { FaTh, FaGlobe, FaLink, FaCog, FaQuestionCircle } from 'react-icons/fa';
import { SettingsDialog } from './SettingsDialog';
import { AppsView, BrowserView, PairingsView } from './views';

type ViewType = 'apps' | 'browser' | 'pairings';

interface NavItem {
  id: ViewType | 'settings' | 'support';
  label: string;
  icon: React.ReactNode;
  onClick: () => void;
}

export const VaultInterface = () => {
  const [currentView, setCurrentView] = useState<ViewType>('browser');
  const { open: isSettingsOpen, onOpen: openSettings, onClose: closeSettings } = useDisclosure();

  const handleSupportClick = () => {
    window.open('https://support.keepkey.com', '_blank');
  };

  const navItems: NavItem[] = [
    {
      id: 'apps',
      label: 'Apps',
      icon: <FaTh />,
      onClick: () => setCurrentView('apps'),
    },
    {
      id: 'browser',
      label: 'Browser',
      icon: <FaGlobe />,
      onClick: () => setCurrentView('browser'),
    },
    {
      id: 'pairings',
      label: 'Pairings',
      icon: <FaLink />,
      onClick: () => setCurrentView('pairings'),
    },
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

  const renderCurrentView = () => {
    switch (currentView) {
      case 'apps':
        return <AppsView />;
      case 'browser':
        return <BrowserView />;
      case 'pairings':
        return <PairingsView />;
      default:
        return <AppsView />;
    }
  };

  return (
    <Box height="100vh" width="100vw" display="flex" flexDirection="column">
      {/* Main Content Area */}
      <Box flex="1" overflow="hidden">
        {renderCurrentView()}
      </Box>

      {/* Bottom Navigation */}
      <Box
        height="80px"
        bg="gray.900"
        borderTop="1px solid"
        borderColor="gray.700"
        px={4}
        py={2}
      >
        <HStack justify="space-around" align="center" height="100%">
          {navItems.map((item) => (
            <Button
              key={item.id}
              variant="ghost"
              size="sm"
              height="60px"
              minWidth="60px"
              flexDirection="column"
              gap={1}
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
              <Box fontSize="lg">{item.icon}</Box>
              <Text fontSize="xs" fontWeight="medium">
                {item.label}
              </Text>
            </Button>
          ))}
        </HStack>
      </Box>

      {/* Settings Modal */}
      <SettingsDialog isOpen={isSettingsOpen} onClose={closeSettings} />
    </Box>
  );
}; 