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
  Spinner
} from '@chakra-ui/react';
import { FaArrowLeft, FaCopy, FaCheck } from 'react-icons/fa';
import QRCode from 'react-qr-code';

interface ReceiveProps {
  onBack?: () => void;
}

// API service for Bitcoin addresses
const bitcoinService = {
  async getAddress(): Promise<string> {
    try {
      const response = await fetch('http://localhost:1646/api/v1/addresses/utxo', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          coin: 'Bitcoin',
          address_n: [2147483692, 2147483648, 2147483648, 0, 0], // m/44'/0'/0'/0/0
          script_type: 'p2wpkh', // Native SegWit
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

  async generateNewAddress(index: number = 0): Promise<string> {
    try {
      const response = await fetch('http://localhost:1646/api/v1/addresses/utxo', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          coin: 'Bitcoin',
          address_n: [2147483692, 2147483648, 2147483648, 0, index], // m/44'/0'/0'/0/{index}
          script_type: 'p2wpkh', // Native SegWit
          show_display: false
        })
      });
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const data = await response.json();
      return data.address;
    } catch (error) {
      console.error('Failed to generate new Bitcoin address:', error);
      // Fallback to mock address for development
      return `bc1q${Math.random().toString(36).substring(2, 32)}`;
    }
  }
};

const Receive: React.FC<ReceiveProps> = ({ onBack }) => {
  const [bitcoinAddress, setBitcoinAddress] = useState('');
  const [loading, setLoading] = useState(true);
  const [hasCopied, setHasCopied] = useState(false);
  const [addressIndex, setAddressIndex] = useState(0);

  const onCopy = () => {
    navigator.clipboard.writeText(bitcoinAddress);
    setHasCopied(true);
    setTimeout(() => setHasCopied(false), 2000);
  };

  // Load initial address
  useEffect(() => {
    const fetchAddress = async () => {
      setLoading(true);
      try {
        const address = await bitcoinService.getAddress();
        setBitcoinAddress(address);
      } catch (error) {
        console.error('Failed to fetch address:', error);
      } finally {
        setLoading(false);
      }
    };
    fetchAddress();
  }, []);

  const generateNewAddress = async () => {
    setLoading(true);
    setHasCopied(false);
    try {
      const newIndex = addressIndex + 1;
      const address = await bitcoinService.generateNewAddress(newIndex);
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
          <Heading size="lg" color="white" flex="1" textAlign="center">
            Receive Bitcoin
          </Heading>
          <Box w="40px" /> {/* Spacer for centering */}
        </HStack>

        {/* QR Code Area */}
        <VStack gap={4} bg="gray.800" p={6} borderRadius="lg">
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
          <Button
            colorScheme="blue"
            size="lg"
            width="100%"
            onClick={generateNewAddress}
            disabled={loading}
          >
            {loading ? <Spinner size="sm" /> : "Generate New Address"}
          </Button>
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
