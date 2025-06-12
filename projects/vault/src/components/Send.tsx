import React, { useState } from 'react';
import { 
  Box, 
  Heading, 
  Text, 
  VStack, 
  HStack, 
  Button, 
  Input, 
  InputGroup, 
  Flex,
  IconButton
} from '@chakra-ui/react';
import { FaArrowLeft, FaQrcode } from 'react-icons/fa';
import { SiBitcoin } from 'react-icons/si';

interface SendProps {
  onBack?: () => void;
}

const Send: React.FC<SendProps> = ({ onBack }) => {
  const [recipientAddress, setRecipientAddress] = useState('');
  const [amount, setAmount] = useState('');
  const [feeRate, setFeeRate] = useState('medium');

  const handleSend = async () => {
    // TODO: Implement actual send functionality
    console.log('Sending Bitcoin:', { recipientAddress, amount, feeRate });
  };

  return (
    <Box height="100%" bg="transparent" display="flex" alignItems="center" justifyContent="center" p={6}>
      <VStack 
        align="stretch" 
        gap={6} 
        maxW="400px" 
        w="100%"
        bg="rgba(26, 32, 44, 0.95)" 
        p={6} 
        borderRadius="xl" 
        backdropFilter="blur(20px)"
        border="1px solid rgba(255, 255, 255, 0.1)"
        boxShadow="2xl"
      >
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
              Send Bitcoin
            </Heading>
          </Flex>
          <Box w="40px" /> {/* Spacer for centering */}
        </HStack>

        {/* Send Form */}
        <VStack gap={4} bg="gray.800" p={6} borderRadius="lg">
          {/* Recipient Address */}
          <Box w="100%">
            <Text color="gray.300" mb={2} fontSize="sm" fontWeight="medium">
              Recipient Address
            </Text>
            <HStack>
              <Input
                placeholder="Enter Bitcoin address"
                value={recipientAddress}
                onChange={(e) => setRecipientAddress(e.target.value)}
                color="white"
                bg="gray.700"
                border="1px solid"
                borderColor="gray.600"
                _focus={{ borderColor: "blue.400" }}
                flex="1"
              />
              <IconButton
                aria-label="Scan QR code"
                size="sm"
                onClick={() => {
                  // TODO: Implement QR code scanning
                  console.log('QR code scanning not implemented yet');
                }}
              >
                <FaQrcode />
              </IconButton>
            </HStack>
          </Box>

          {/* Amount */}
          <Box w="100%">
            <Text color="gray.300" mb={2} fontSize="sm" fontWeight="medium">
              Amount (BTC)
            </Text>
            <Input
              placeholder="0.00000000"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              color="white"
              bg="gray.700"
              border="1px solid"
              borderColor="gray.600"
              _focus={{ borderColor: "blue.400" }}
              type="number"
              step="0.00000001"
            />
          </Box>

          {/* Fee Selection */}
          <Box w="100%">
            <Text color="gray.300" mb={2} fontSize="sm" fontWeight="medium">
              Transaction Fee
            </Text>
            <HStack gap={2}>
              {['slow', 'medium', 'fast'].map((rate) => (
                <Button
                  key={rate}
                  size="sm"
                  variant={feeRate === rate ? "solid" : "outline"}
                  colorScheme={feeRate === rate ? "blue" : "gray"}
                  onClick={() => setFeeRate(rate)}
                  flex="1"
                  textTransform="capitalize"
                >
                  {rate}
                </Button>
              ))}
            </HStack>
            <Text fontSize="xs" color="gray.500" mt={1}>
              Estimated fee: ~$0.50 USD
            </Text>
          </Box>
        </VStack>

        {/* Action Buttons */}
        <VStack gap={3}>
          <Button
            colorScheme="blue"
            size="lg"
            width="100%"
            onClick={handleSend}
            disabled={!recipientAddress || !amount}
          >
            Send Bitcoin
          </Button>
          <Button
            variant="ghost"
            color="gray.400"
            onClick={onBack}
          >
            Cancel
          </Button>
        </VStack>
      </VStack>
    </Box>
  );
};

export default Send;
