import React, { useState } from 'react';
import { 
  Box, 
  Heading, 
  Text, 
  VStack, 
  HStack, 
  Button, 
  IconButton,
  Flex,
  Spinner
} from '@chakra-ui/react';
import { FaArrowLeft, FaCopy, FaCheck, FaEye } from 'react-icons/fa';
import { SiBitcoin } from 'react-icons/si';
import QRCode from 'react-qr-code';
import { useWallet } from '../contexts/WalletContext';

interface ReceiveProps {
  onBack?: () => void;
}

const Receive: React.FC<ReceiveProps> = ({ onBack }) => {
  const { portfolio, getReceiveAddress, selectAsset, loading: walletLoading } = useWallet();
  
  const [address, setAddress] = useState<string>('');
  const [hasCopied, setHasCopied] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Get BTC asset from portfolio
  const btcAsset = portfolio?.assets.find(asset => asset.symbol === 'BTC');

  // Generate receive address
  const generateAddress = async () => {
    if (!btcAsset) {
      setError('No Bitcoin asset found in portfolio');
      return;
    }

    try {
      setLoading(true);
      setError(null);
      
      // Select the BTC asset first
      selectAsset(btcAsset);
      
      // Get receive address from wallet context
      const receiveAddress = await getReceiveAddress();
      
      if (receiveAddress) {
        setAddress(receiveAddress);
      } else {
        throw new Error('Failed to generate receive address');
      }
      
    } catch (error) {
      console.error('Error generating address:', error);
      setError(error instanceof Error ? error.message : 'Failed to generate address');
    } finally {
      setLoading(false);
    }
  };

  // Copy address to clipboard
  const onCopy = () => {
    if (address) {
      navigator.clipboard.writeText(address);
      setHasCopied(true);
      setTimeout(() => setHasCopied(false), 2000);
    }
  };

  if (walletLoading) {
    return (
      <Box height="100%" display="flex" alignItems="center" justifyContent="center" bg="transparent">
        <VStack gap={4}>
          <Spinner size="xl" color="blue.400" />
          <Text color="gray.300" fontSize="lg">Loading Wallet...</Text>
        </VStack>
      </Box>
    );
  }

  return (
    <Box height="100%" bg="transparent" display="flex" alignItems="center" justifyContent="center" p={6}>
      <Box 
        maxW="500px" 
        w="100%"
        bg="rgba(26, 32, 44, 0.95)" 
        p={6} 
        borderRadius="xl" 
        backdropFilter="blur(20px)"
        border="1px solid rgba(255, 255, 255, 0.1)"
        boxShadow="2xl"
      >
        <VStack align="stretch" gap={6}>
          {/* Header */}
          <HStack>
            <IconButton
              aria-label="Go back"
              onClick={onBack}
              size="sm"
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

          {/* Main Content */}
          <VStack gap={6} bg="gray.800" p={6} borderRadius="lg" minH="400px" justify="center">
            {!address ? (
              /* Generate Address View */
              <VStack gap={4} textAlign="center">
                <Box color="orange.400" fontSize="4xl">
                  <SiBitcoin />
                </Box>
                
                <Text color="gray.300" fontSize="lg" fontWeight="medium">
                  Generate Receive Address
                </Text>
                
                <Text color="gray.400" fontSize="sm" textAlign="center" maxW="300px">
                  Generate a Bitcoin address to receive payments. This address will be linked to your KeepKey device.
                </Text>

                {error && (
                  <Box bg="red.900" p={3} borderRadius="md" border="1px solid" borderColor="red.600">
                    <Text color="red.200" fontSize="sm">⚠️ {error}</Text>
                  </Box>
                )}

                <Button
                  colorScheme="blue"
                  size="lg"
                  onClick={generateAddress}
                  disabled={loading || !btcAsset}
                  minW="200px"
                >
                  <HStack gap={2}>
                    {loading ? <Spinner size="sm" /> : <FaEye />}
                    <Text>{loading ? 'Generating...' : 'Generate Address'}</Text>
                  </HStack>
                </Button>

                {!btcAsset && (
                  <Text color="yellow.400" fontSize="xs" textAlign="center">
                    No Bitcoin asset found in portfolio
                  </Text>
                )}
              </VStack>
            ) : (
              /* Address Display View */
              <VStack gap={4} w="100%">
                {/* Success Header */}
                <VStack gap={2} textAlign="center">
                  <Text color="green.300" fontSize="lg" fontWeight="bold">
                    ✅ Receive Address Generated
                  </Text>
                  <Text color="gray.400" fontSize="sm">
                    Use this address to receive Bitcoin payments
                  </Text>
                </VStack>

                {/* QR Code */}
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
                  <QRCode
                    value={address}
                    size={184}
                    style={{ height: "auto", maxWidth: "100%", width: "100%" }}
                    viewBox={`0 0 184 184`}
                  />
                </Box>

                {/* Address String */}
                <VStack w="100%" gap={2}>
                  <Text color="gray.300" fontSize="sm" fontWeight="medium">
                    Bitcoin Address
                  </Text>
                  <Box
                    w="100%"
                    bg="gray.700"
                    p={3}
                    borderRadius="md"
                    border="1px solid"
                    borderColor="gray.600"
                  >
                    <Text
                      color="white"
                      fontSize="sm"
                      fontFamily="mono"
                      wordBreak="break-all"
                      textAlign="center"
                    >
                      {address}
                    </Text>
                  </Box>
                </VStack>

                {/* Action Buttons */}
                <HStack gap={3} w="100%">
                  <Button
                    colorScheme="blue"
                    size="lg"
                    onClick={onCopy}
                    flex="1"
                    bg={hasCopied ? "green.600" : "blue.600"}
                    _hover={{ bg: hasCopied ? "green.500" : "blue.500" }}
                  >
                    <HStack gap={2}>
                      {hasCopied ? <FaCheck /> : <FaCopy />}
                      <Text>{hasCopied ? 'Copied!' : 'Copy Address'}</Text>
                    </HStack>
                  </Button>
                  
                  <Button
                    variant="outline"
                    colorScheme="gray"
                    size="lg"
                    onClick={() => {
                      setAddress('');
                      setError(null);
                    }}
                    flex="1"
                  >
                    Generate New
                  </Button>
                </HStack>

                {/* Security Notice */}
                <Box bg="blue.900" p={3} borderRadius="md" border="1px solid" borderColor="blue.600">
                  <Text color="blue.200" fontSize="xs" textAlign="center">
                    🔒 This address was generated by your KeepKey device and is safe to use for receiving Bitcoin.
                  </Text>
                </Box>
              </VStack>
            )}
          </VStack>

          {/* Back Button */}
          <Button
            variant="ghost"
            color="gray.400"
            onClick={onBack}
            alignSelf="center"
          >
            Back to Vault
          </Button>
        </VStack>
      </Box>
    </Box>
  );
};

export default Receive; 