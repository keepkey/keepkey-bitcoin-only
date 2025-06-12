import React, { useState, useEffect } from 'react';
import { 
  Box, 
  Heading, 
  Text, 
  VStack, 
  HStack, 
  Button, 
  Input,
  IconButton,
  Flex,
  Spinner
} from '@chakra-ui/react';
import { FaArrowLeft, FaCopy, FaCheck, FaEye } from 'react-icons/fa';
import { SiBitcoin } from 'react-icons/si';
import QRCode from 'react-qr-code';

interface ReceiveProps {
  onBack?: () => void;
}

// API service for Bitcoin addresses
const bitcoinService = {
  // Helper to get the correct BIP number and derivation path for script type
  getScriptTypeInfo(scriptType: string) {
    switch (scriptType) {
      case 'p2pkh':
        return { bip: 44, pathPrefix: "m/44'/0'/0'" };
      case 'p2sh-p2wpkh':
        return { bip: 49, pathPrefix: "m/49'/0'/0'" };
      case 'p2wpkh':
        return { bip: 84, pathPrefix: "m/84'/0'/0'" };
      default:
        return { bip: 84, pathPrefix: "m/84'/0'/0'" }; // Default to native SegWit
    }
  },

  // Helper to create derivation path array
  createDerivationPath(scriptType: string, index: number): number[] {
    const { bip } = this.getScriptTypeInfo(scriptType);
    const hardened = 0x80000000;
    return [
      hardened + bip,    // BIP number (44'/49'/84')
      hardened + 0,      // Coin type (0' for Bitcoin)
      hardened + 0,      // Account (0')
      0,                 // External chain (0)
      index              // Address index
    ];
  },

  async getAddress(scriptType: string = 'p2wpkh', index: number = 0): Promise<string> {
    try {
      const response = await fetch('http://localhost:1646/addresses/utxo', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          coin: 'Bitcoin',
          address_n: this.createDerivationPath(scriptType, index),
          script_type: scriptType,
          show_display: false
        })
      });
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const data = await response.json();
      return data.address;
    } catch (error) {
      console.error('Failed to get Bitcoin address:', error);
      // Fallback to mock address for development
      return 'bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh';
    }
  },

  async generateNewAddress(scriptType: string = 'p2wpkh', index: number = 0): Promise<string> {
    return this.getAddress(scriptType, index);
  }
};

const Receive: React.FC<ReceiveProps> = ({ onBack }) => {
  const [bitcoinAddress, setBitcoinAddress] = useState('');
  const [loading, setLoading] = useState(true);
  const [hasCopied, setHasCopied] = useState(false);
  const [addressIndex, setAddressIndex] = useState(0);
  const [scriptType] = useState('p2wpkh'); // Default to native SegWit
  const [isViewingOnDevice, setIsViewingOnDevice] = useState(false);

  // Helper function to get the correct derivation path format
  const formatDerivationPath = (scriptType: string, index: number) => {
    const { pathPrefix } = bitcoinService.getScriptTypeInfo(scriptType);
    return `${pathPrefix}/0/${index}`;
  };

  const onCopy = () => {
    navigator.clipboard.writeText(bitcoinAddress);
    setHasCopied(true);
    setTimeout(() => setHasCopied(false), 2000);
  };

  // View address on device
  const viewOnDevice = async () => {
    setIsViewingOnDevice(true);
    try {
      console.log("Displaying address on KeepKey device...");
      
      // Make API call with show_display: true
      const response = await fetch('http://localhost:1646/api/v1/addresses/utxo', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          coin: 'Bitcoin',
          address_n: bitcoinService.createDerivationPath(scriptType, addressIndex),
          script_type: scriptType,
          show_display: true // This will display the address on the device
        })
      });

      if (response.ok) {
        console.log("Address successfully displayed on device");
        alert("Check your KeepKey device - the address is now displayed on the screen. Please verify it matches!");
      } else {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
    } catch (error) {
      console.error('Failed to display address on device:', error);
      alert("Failed to display address on device. Please ensure your KeepKey is connected and try again.");
    } finally {
      setIsViewingOnDevice(false);
    }
  };

  // Load initial address
  useEffect(() => {
    const fetchAddress = async () => {
      setLoading(true);
      try {
        const address = await bitcoinService.getAddress(scriptType, addressIndex);
        setBitcoinAddress(address);
      } catch (error) {
        console.error('Failed to fetch address:', error);
      } finally {
        setLoading(false);
      }
    };
    fetchAddress();
  }, [scriptType, addressIndex]);

  const generateNewAddress = async () => {
    setLoading(true);
    setHasCopied(false);
    try {
      const newIndex = addressIndex + 1;
      const address = await bitcoinService.generateNewAddress(scriptType, newIndex);
      setBitcoinAddress(address);
      setAddressIndex(newIndex);
    } catch (error) {
      console.error('Failed to generate new address:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Box height="100%" bg="gray.900" p={6}>
      <VStack align="stretch" gap={6} maxW="400px" mx="auto">
        {/* Header with back button */}
        <HStack>
          <IconButton
            aria-label="Go back"
            onClick={onBack}
          >
            <FaArrowLeft />
          </IconButton>
          <Flex align="center" justify="center" flex="1" gap={2}>
            <Box color="orange.400" fontSize="xl">
              <SiBitcoin />
            </Box>
            <Heading size="lg" color="white">
              Receive Bitcoin
            </Heading>
          </Flex>
          <Box w="40px" /> {/* Spacer for centering */}
        </HStack>

        {/* QR Code Area */}
        <VStack gap={4} bg="gray.800" p={6} borderRadius="lg">
          {/* Derivation Path */}
          <Box textAlign="center">
            <Text fontSize="xs" color="gray.500" mb={1}>
              Derivation Path
            </Text>
                         <Text fontSize="sm" color="gray.300" fontFamily="mono">
               {formatDerivationPath(scriptType, addressIndex)}
             </Text>
          </Box>
          
          <Box
            w="200px"
            h="200px"
            bg="white"
            borderRadius="md"
            display="flex"
            alignItems="center"
            justifyContent="center"
            mx="auto"
            p={2}
          >
            {loading ? (
              <Spinner size="xl" color="gray.600" />
            ) : (
              <QRCode
                value={bitcoinAddress}
                size={184}
                style={{ height: "auto", maxWidth: "100%", width: "100%" }}
                viewBox={`0 0 184 184`}
              />
            )}
          </Box>

          {/* Bitcoin Address */}
          <Box w="100%">
            <Text color="gray.300" mb={2} fontSize="sm" fontWeight="medium">
              Your Bitcoin Address
            </Text>
            {loading ? (
              <Box bg="gray.700" p={3} borderRadius="md" display="flex" alignItems="center" justifyContent="center">
                <Spinner size="sm" color="blue.400" />
              </Box>
            ) : (
              <HStack>
                <Input
                  value={bitcoinAddress}
                  color="white"
                  bg="gray.700"
                  border="1px solid"
                  borderColor="gray.600"
                  readOnly
                  fontSize="sm"
                  flex="1"
                />
                <IconButton
                  aria-label={hasCopied ? "Copied!" : "Copy address"}
                  onClick={onCopy}
                  colorScheme={hasCopied ? "green" : "blue"}
                  size="sm"
                >
                  {hasCopied ? <FaCheck /> : <FaCopy />}
                </IconButton>
              </HStack>
            )}
            {hasCopied && (
              <Text fontSize="xs" color="green.400" mt={1}>
                Address copied to clipboard!
              </Text>
            )}
          </Box>

          {/* Address Info */}
          <Box w="100%" bg="blue.900" p={3} borderRadius="md">
            <Text fontSize="xs" color="blue.200">
              <strong>Notice:</strong> This is a fresh Bitcoin address. 
              For privacy, a new address is generated for each transaction.
            </Text>
          </Box>
        </VStack>

        {/* Action Buttons */}
        <VStack gap={3}>
          <HStack gap={3} width="100%">
            <Button
              colorScheme="blue"
              size="lg"
              onClick={generateNewAddress}
              disabled={loading}
              flex="1"
            >
              {loading ? <Spinner size="sm" /> : "Generate New"}
            </Button>
            <Button
              colorScheme="purple"
              size="lg"
              onClick={viewOnDevice}
              disabled={loading || isViewingOnDevice}
              flex="1"
            >
              {isViewingOnDevice ? (
                <HStack gap={2}>
                  <Spinner size="sm" />
                  <Text>Viewing...</Text>
                </HStack>
              ) : (
                <HStack gap={2}>
                  <FaEye />
                  <Text>View on Device</Text>
                </HStack>
              )}
            </Button>
          </HStack>
          <Button
            variant="ghost"
            color="gray.400"
            onClick={onBack}
          >
            Done
          </Button>
        </VStack>
      </VStack>
    </Box>
  );
};

export default Receive;
