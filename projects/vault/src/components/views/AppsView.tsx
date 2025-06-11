import { Box, Grid, GridItem, Text, Button, Stack, Icon, Flex } from '@chakra-ui/react';
import { FaTh, FaPlus, FaRocket, FaExchangeAlt, FaWallet } from 'react-icons/fa';
import { invoke } from '@tauri-apps/api/core';

interface App {
  id: string;
  name: string;
  description: string;
  icon: React.ReactNode;
  url?: string;
  category: 'defi' | 'exchange' | 'wallet' | 'other';
}

const defaultApps: App[] = [
  {
    id: 'shapeshift',
    name: 'ShapeShift',
    description: 'Decentralized trading platform',
    icon: <FaExchangeAlt />,
    url: 'https://app.shapeshift.com',
    category: 'defi',
  },
  {
    id: 'metamask',
    name: 'MetaMask',
    description: 'Ethereum wallet and gateway',
    icon: <FaWallet />,
    url: 'https://metamask.io',
    category: 'wallet',
  },
  {
    id: 'uniswap',
    name: 'Uniswap',
    description: 'Decentralized exchange',
    icon: <FaRocket />,
    url: 'https://app.uniswap.org',
    category: 'defi',
  },
];

export const AppsView = () => {
  const handleAppClick = async (app: App) => {
    if (app.url) {
      try {
        // Tell backend to switch to browser and navigate to app URL
        await invoke('vault_open_app', { 
          appId: app.id, 
          appName: app.name, 
          url: app.url 
        });
      } catch (error) {
        console.error('Failed to open app via backend:', error);
        // Fallback to direct open
        window.open(app.url, '_blank');
      }
    }
  };

  const handleAddApp = () => {
    // TODO: Implement add custom app functionality
    console.log('Add custom app clicked');
  };

  return (
    <Box p={6} height="100%" bg="rgba(0, 0, 0, 0.4)">
      <Stack direction="column" gap={6} align="stretch" height="100%">
        {/* Header */}
        <Flex justify="space-between" align="center">
          <Stack direction="column" gap={1} align="start">
            <Text fontSize="2xl" fontWeight="bold" color="white">
              Apps
            </Text>
            <Text fontSize="md" color="gray.400">
              Connect to your favorite dApps and services
            </Text>
          </Stack>
          <Button
            colorScheme="blue"
            variant="outline"
            size="sm"
            onClick={handleAddApp}
          >
            <Icon as={FaPlus} mr={2} />
            Add App
          </Button>
        </Flex>

        {/* Apps Grid */}
        <Box flex="1" overflowY="auto">
          <Grid templateColumns="repeat(auto-fill, minmax(200px, 1fr))" gap={4}>
            {defaultApps.map((app) => (
              <GridItem key={app.id}>
                <Button
                  height="120px"
                  width="100%"
                  bg="gray.800"
                  border="1px solid"
                  borderColor="gray.700"
                  _hover={{
                    bg: "gray.700",
                    borderColor: "blue.500",
                    transform: "translateY(-2px)",
                  }}
                  _active={{
                    bg: "gray.600",
                  }}
                  borderRadius="lg"
                  onClick={() => handleAppClick(app)}
                  transition="all 0.2s"
                >
                  <Stack direction="column" gap={3}>
                    <Box
                      p={3}
                      borderRadius="md"
                      bg="blue.500"
                      color="white"
                      fontSize="2xl"
                    >
                      {app.icon}
                    </Box>
                    <Stack direction="column" gap={1}>
                      <Text fontWeight="semibold" color="white" fontSize="sm">
                        {app.name}
                      </Text>
                      <Text fontSize="xs" color="gray.400" textAlign="center">
                        {app.description}
                      </Text>
                    </Stack>
                  </Stack>
                </Button>
              </GridItem>
            ))}
          </Grid>
        </Box>
      </Stack>
    </Box>
  );
}; 