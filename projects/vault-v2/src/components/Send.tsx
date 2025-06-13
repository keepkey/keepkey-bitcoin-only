import React, { useState, useEffect } from 'react';
import { 
  Box, 
  Heading, 
  Text, 
  VStack, 
  HStack, 
  Button, 
  Input, 
  Flex,
  IconButton,
  Spinner
} from '@chakra-ui/react';
import { FaArrowLeft, FaQrcode, FaPaperPlane } from 'react-icons/fa';
import { SiBitcoin } from 'react-icons/si';
import { useWallet } from '../contexts/WalletContext';

interface SendPageProps {
  onBack: () => void;
}

const Send: React.FC<SendPageProps> = ({ onBack }) => {
  const { portfolio, sendAsset, loading: walletLoading, error: walletError } = useWallet();
  
  // State according to planning document 
  const [recipientAddress, setRecipientAddress] = useState('');
  const [amount, setAmount] = useState('');
  const [feeRate, setFeeRate] = useState<'slow' | 'medium' | 'fast'>('medium');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [addressValidation, setAddressValidation] = useState<{ valid: boolean; error?: string }>({ valid: false });
  const [amountCurrency, setAmountCurrency] = useState<'BTC' | 'USD'>('BTC');
  const [btcPrice, setBtcPrice] = useState<number>(43000);

  // Get BTC asset from portfolio
  const btcAsset = portfolio?.assets.find(asset => asset.symbol === 'BTC');
  const availableBalance = btcAsset ? parseFloat(btcAsset.balance) : 0;

  // Load initial data
  useEffect(() => {
    // Set BTC price from portfolio if available
    if (btcAsset && btcAsset.price_usd) {
      setBtcPrice(btcAsset.price_usd);
    }
  }, [btcAsset]);

  // Validate address when it changes
  useEffect(() => {
    if (recipientAddress) {
      // Simple Bitcoin address validation (basic check)
      const isValidAddress = /^[13][a-km-zA-HJ-NP-Z1-9]{25,34}$|^bc1[a-z0-9]{39,59}$/.test(recipientAddress);
      setAddressValidation({ 
        valid: isValidAddress,
        error: isValidAddress ? undefined : 'Invalid Bitcoin address format'
      });
    } else {
      setAddressValidation({ valid: false });
    }
  }, [recipientAddress]);

  const handleMaxAmount = () => {
    if (availableBalance > 0) {
      // Reserve some amount for fees (simplified)
      const maxAmount = Math.max(0, availableBalance - 0.0001); // Reserve 0.0001 BTC for fees
      setAmount(maxAmount.toFixed(8));
    }
  };

  const handleSend = async () => {
    if (!recipientAddress || !amount || !addressValidation.valid) {
      setError('Please enter a valid recipient address and amount');
      return;
    }

    const sendAmount = parseFloat(amount);
    if (sendAmount <= 0) {
      setError('Amount must be greater than 0');
      return;
    }

    if (sendAmount > availableBalance) {
      setError('Insufficient balance');
      return;
    }

    try {
      setLoading(true);
      setError(null);
      setSuccess(null);

      // Use WalletContext to send
      const success = await sendAsset(recipientAddress, amount);
      
      if (success) {
        setSuccess(`Transaction sent successfully! Amount: ${amount} BTC to ${recipientAddress.substring(0, 20)}...`);
        
        // Reset form
        setRecipientAddress('');
        setAmount('');
      } else {
        throw new Error('Transaction failed');
      }

    } catch (error) {
      console.error('Error sending transaction:', error);
      setError(error instanceof Error ? error.message : 'Unknown error occurred');
    } finally {
      setLoading(false);
    }
  };

  // Currency conversion helpers
  const convertBtcToUsd = (btc: number): number => btc * btcPrice;
  const convertUsdToBtc = (usd: number): number => usd / btcPrice;
  
  const formatCurrency = (value: number, currency: 'BTC' | 'USD'): string => {
    if (currency === 'BTC') {
      return value.toFixed(8);
    } else {
      return value.toFixed(2);
    }
  };

  const toggleCurrency = () => {
    const currentAmountNum = parseFloat(amount) || 0;
    let convertedAmount: number;
    
    if (amountCurrency === 'BTC') {
      // Converting from BTC to USD
      convertedAmount = convertBtcToUsd(currentAmountNum);
      setAmountCurrency('USD');
    } else {
      // Converting from USD to BTC
      convertedAmount = convertUsdToBtc(currentAmountNum);
      setAmountCurrency('BTC');
    }
    
    setAmount(formatCurrency(convertedAmount, amountCurrency === 'BTC' ? 'USD' : 'BTC'));
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
      <VStack 
        align="stretch" 
        gap={6} 
        maxW="500px" 
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
            size="sm"
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

        {/* Balance Display */}
        <Box bg="gray.800" p={4} borderRadius="lg" textAlign="center">
          <Text color="gray.400" fontSize="sm">Available Balance</Text>
          <Text color="white" fontSize="2xl" fontWeight="bold">
            {availableBalance.toFixed(8)} BTC
          </Text>
          <Text color="gray.400" fontSize="sm">
            ≈ ${convertBtcToUsd(availableBalance).toFixed(2)} USD
          </Text>
        </Box>

        {/* Send Form */}
        <VStack gap={6} bg="gray.800" p={6} borderRadius="lg">
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
                borderColor={addressValidation.valid ? "green.400" : "gray.600"}
                _focus={{ borderColor: addressValidation.valid ? "green.400" : "blue.400" }}
                flex="1"
              />
              <IconButton
                aria-label="Scan QR code"
                size="md"
                onClick={() => {
                  alert('QR code scanning will be implemented later');
                }}
              >
                <FaQrcode />
              </IconButton>
            </HStack>
            {!addressValidation.valid && recipientAddress && (
              <Text color="red.400" fontSize="xs" mt={1}>
                {addressValidation.error || 'Invalid Bitcoin address'}
              </Text>
            )}
            {addressValidation.valid && (
              <Text color="green.400" fontSize="xs" mt={1}>
                ✓ Valid Bitcoin address
              </Text>
            )}
          </Box>

          {/* Amount with BTC/USD Toggle */}
          <Box w="100%">
            <HStack justify="space-between" mb={2}>
              <HStack gap={2}>
                <Text color="gray.300" fontSize="sm" fontWeight="medium">
                  Amount
                </Text>
                <Button
                  size="xs"
                  variant="ghost"
                  color="blue.400"
                  onClick={toggleCurrency}
                  fontWeight="bold"
                  _hover={{ color: "blue.300" }}
                >
                  ({amountCurrency})
                </Button>
              </HStack>
              <Button
                size="xs"
                variant="outline"
                colorScheme="blue"
                onClick={handleMaxAmount}
                disabled={availableBalance === 0}
              >
                Max
              </Button>
            </HStack>
            <Input
              placeholder={amountCurrency === 'BTC' ? "0.00000000" : "0.00"}
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              color="white"
              bg="gray.700"
              border="1px solid"
              borderColor="gray.600"
              _focus={{ borderColor: "blue.400" }}
              type="number"
              step={amountCurrency === 'BTC' ? "0.00000001" : "0.01"}
            />
            <Text fontSize="xs" color="gray.500" mt={1}>
              {amount && parseFloat(amount) > 0 && (
                <>
                  ≈ {amountCurrency === 'BTC' 
                    ? `$${convertBtcToUsd(parseFloat(amount)).toFixed(2)} USD` 
                    : `${convertUsdToBtc(parseFloat(amount)).toFixed(8)} BTC`
                  }
                </>
              )}
            </Text>
          </Box>

          {/* Fee Selection */}
          <Box w="100%">
            <Text color="gray.300" mb={2} fontSize="sm" fontWeight="medium">
              Transaction Fee
            </Text>
            <HStack gap={1}>
              {(['slow', 'medium', 'fast'] as const).map((preset) => {
                const feeRates = { slow: 1, medium: 5, fast: 10 };
                const presetFeeRate = feeRates[preset];
                
                return (
                  <Button
                    key={preset}
                    size="xs"
                    variant={feeRate === preset ? "solid" : "outline"}
                    colorScheme={feeRate === preset ? "blue" : "gray"}
                    onClick={() => setFeeRate(preset)}
                    flex="1"
                    textTransform="capitalize"
                    py={3}
                    px={2}
                  >
                    <VStack gap={0} align="center">
                      <Text fontSize="xs" fontWeight="bold">{preset}</Text>
                      <Text fontSize="2xs" opacity={0.8}>{presetFeeRate} sat/vB</Text>
                    </VStack>
                  </Button>
                );
              })}
            </HStack>
          </Box>

          {/* Error Display */}
          {(error || walletError) && (
            <Box bg="red.900" p={3} borderRadius="md" border="1px solid" borderColor="red.600">
              <Text color="red.200" fontSize="sm">⚠️ {error || walletError}</Text>
            </Box>
          )}

          {/* Success Display */}
          {success && (
            <Box bg="green.900" p={3} borderRadius="md" border="1px solid" borderColor="green.600">
              <Text color="green.200" fontSize="sm">✅ {success}</Text>
            </Box>
          )}
        </VStack>

        {/* Action Buttons */}
        <VStack gap={3}>
          <Button
            colorScheme="blue"
            size="lg"
            width="100%"
            onClick={handleSend}
            disabled={!addressValidation.valid || !amount || loading || !btcAsset}
          >
            <HStack gap={2}>
              {loading ? <Spinner size="sm" /> : <FaPaperPlane />}
              <Text>{loading ? 'Sending...' : 'Send Bitcoin'}</Text>
            </HStack>
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