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
import { PioneerAPI } from '../lib/api';

interface SendPageProps {
  onBack: () => void;
}

interface FeeRates {
  slow: number;
  medium: number;
  fast: number;
}

const Send: React.FC<SendPageProps> = ({ onBack }) => {
  const { portfolio, sendAsset, loading: walletLoading, error: walletError, selectAsset } = useWallet();
  
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
  
  // Fee-related state
  const [feeRates, setFeeRates] = useState<FeeRates>({ slow: 1, medium: 5, fast: 10 }); // Fallback rates
  const [loadingFees, setLoadingFees] = useState(true);
  const [feeError, setFeeError] = useState<string | null>(null);

  // Get BTC balance from portfolio (sum all BTC assets like Portfolio component does)
  const btcAssets = portfolio?.assets.filter(asset => 
    asset.caip === 'bip122:000000000019d6689c085ae165831e93/slip44:0'
  ) || [];
  const availableBalance = btcAssets.reduce((sum, asset) => sum + parseFloat(asset.balance), 0);
  
  // Debug logging to help troubleshoot balance issues
  console.log('🔍 Send component balance debug:', {
    walletLoading,
    portfolio: portfolio ? 'loaded' : 'null',
    totalAssets: portfolio?.assets.length || 0,
    allAssetCAIPs: portfolio?.assets.map(a => a.caip) || [],
    btcAssets: btcAssets.length,
    availableBalance,
    btcAssetsDetails: btcAssets.map(a => ({ caip: a.caip, balance: a.balance, symbol: a.symbol })),
    expectedCAIP: 'bip122:000000000019d6689c085ae165831e93/slip44:0'
  });
  
  // Get first BTC asset for price info (they should all have same price)
  const btcAsset = btcAssets[0] || null;

  // Load fee rates from Pioneer API
  useEffect(() => {
    const loadFeeRates = async () => {
      if (btcAssets.length === 0) return;
      
      try {
        setLoadingFees(true);
        setFeeError(null);
        
        console.log('💰 Loading real fee rates from Pioneer for:', btcAssets[0].caip);
        const feeData = await PioneerAPI.getFeeRates(btcAssets[0].caip);
        
        const realFeeRates: FeeRates = {
          slow: feeData.average || 1,
          medium: feeData.fast || 5,
          fast: feeData.fastest || 10
        };
        
        setFeeRates(realFeeRates);
        console.log('✅ Real fee rates loaded:', realFeeRates);
        
      } catch (error) {
        console.warn('⚠️ Failed to load real fee rates, using fallback:', error);
        setFeeError('Unable to load current fee rates, using estimates');
        // Keep fallback rates
      } finally {
        setLoadingFees(false);
      }
    };

    loadFeeRates();
  }, [btcAssets.length]);

  // Load initial data
  useEffect(() => {
    // Set BTC price from portfolio if available
    if (btcAsset && btcAsset.price_usd) {
      console.log('🔍 Setting BTC price from portfolio:', btcAsset.price_usd);
      setBtcPrice(btcAsset.price_usd);
    } else {
      console.log('🔍 BTC price not available, using default:', btcPrice);
    }
    
    // Ensure BTC asset is selected for sending
    if (btcAssets.length > 0 && btcAsset) {
      selectAsset(btcAsset);
    }
  }, [btcAsset, btcAssets.length, selectAsset]);

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
      // Calculate more accurate fee estimate based on selected fee rate
      const selectedFeeRate = feeRates[feeRate];
      const estimatedFeeInBtc = (250 * selectedFeeRate) / 100000000; // Assume ~250 vBytes for typical tx
      const maxAmountInBtc = Math.max(0, availableBalance - estimatedFeeInBtc);
      
      // Convert to display currency
      if (amountCurrency === 'USD') {
        const maxAmountInUsd = convertBtcToUsd(maxAmountInBtc);
        setAmount(maxAmountInUsd.toFixed(2));
      } else {
        setAmount(maxAmountInBtc.toFixed(8));
      }
    }
  };

  const handleSend = async () => {
    if (!recipientAddress || !amount || !addressValidation.valid) {
      setError('Please enter a valid recipient address and amount');
      return;
    }

    const inputAmount = parseFloat(amount);
    if (inputAmount <= 0) {
      setError('Amount must be greater than 0');
      return;
    }

    // Convert to BTC for validation if in USD mode
    const sendAmountInBtc = amountCurrency === 'USD' ? convertUsdToBtc(inputAmount) : inputAmount;
    
    console.log('🔍 Send validation debug:', {
      inputAmount,
      amountCurrency,
      sendAmountInBtc,
      availableBalance,
      btcPrice,
      wouldPass: sendAmountInBtc <= availableBalance
    });
    
    if (sendAmountInBtc > availableBalance) {
      const availableInDisplayCurrency = amountCurrency === 'USD' 
        ? convertBtcToUsd(availableBalance) 
        : availableBalance;
      setError(`Insufficient balance. Available: ${availableInDisplayCurrency.toFixed(amountCurrency === 'USD' ? 2 : 8)} ${amountCurrency}`);
      return;
    }

    try {
      setLoading(true);
      setError(null);
      setSuccess(null);

      // Use WalletContext to send (always pass BTC amount)
      const btcAmountToSend = amountCurrency === 'USD' ? convertUsdToBtc(inputAmount) : inputAmount;
      await sendAsset(recipientAddress, btcAmountToSend.toFixed(8));
      
      setSuccess(`Transaction sent successfully! Amount: ${amount} ${amountCurrency} to ${recipientAddress.substring(0, 20)}...`);
      
      // Reset form
      setRecipientAddress('');
      setAmount('');

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
          {/* Show balance warning if zero */}
          {availableBalance === 0 && (
            <Text color="yellow.400" fontSize="xs" mt={2}>
              ⚠️ No balance available. Please ensure your wallet is synced.
            </Text>
          )}
          {/* Show currency info */}
          <Text color="gray.500" fontSize="xs" mt={1}>
            Enter amount in {amountCurrency}. Click {amountCurrency} button to switch currency.
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
                  variant="solid"
                  colorScheme={amountCurrency === 'BTC' ? 'orange' : 'green'}
                  onClick={toggleCurrency}
                  fontWeight="bold"
                  _hover={{ opacity: 0.8 }}
                >
                  {amountCurrency}
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
              min="0"
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

          {/* Fee Selection - Now with real rates! */}
          <Box w="100%">
            <HStack justify="space-between" align="center" mb={2}>
              <Text color="gray.300" fontSize="sm" fontWeight="medium">
                Transaction Fee
              </Text>
              {loadingFees && (
                <HStack gap={1}>
                  <Spinner size="xs" color="blue.400" />
                  <Text fontSize="xs" color="gray.500">Loading rates...</Text>
                </HStack>
              )}
              {feeError && (
                <Text fontSize="xs" color="yellow.400">Using estimates</Text>
              )}
              {!loadingFees && !feeError && (
                <Text fontSize="xs" color="green.400">✓ Live rates</Text>
              )}
            </HStack>
            <HStack gap={1}>
              {(['slow', 'medium', 'fast'] as const).map((preset) => {
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
                    disabled={loadingFees}
                  >
                    <VStack gap={0} align="center">
                      <Text fontSize="xs" fontWeight="bold">{preset}</Text>
                      <Text fontSize="2xs" opacity={0.8}>
                        {loadingFees ? '...' : `${Math.round(presetFeeRate)} sat/vB`}
                      </Text>
                    </VStack>
                  </Button>
                );
              })}
            </HStack>
            {/* Fee estimate in BTC/USD */}
            {amount && parseFloat(amount) > 0 && !loadingFees && (
              <Text fontSize="xs" color="gray.500" mt={2} textAlign="center">
                Estimated fee: ~{((250 * feeRates[feeRate]) / 100000000).toFixed(8)} BTC 
                (≈${((250 * feeRates[feeRate]) / 100000000 * btcPrice).toFixed(2)})
              </Text>
            )}
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
            disabled={!addressValidation.valid || !amount || loading || btcAssets.length === 0 || availableBalance === 0}
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