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
import { 
  UTXO, 
  FeeEstimate, 
  SendPageProps
} from '../types/send';
import { sendService } from '../services/sendService';

const Send: React.FC<SendPageProps> = ({ onBack }) => {
  // State according to planning document 
  const [recipientAddress, setRecipientAddress] = useState('');
  const [amount, setAmount] = useState('');
  const [feeRate, setFeeRate] = useState<'slow' | 'medium' | 'fast'>('medium');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [addressValidation, setAddressValidation] = useState<{ valid: boolean; error?: string }>({ valid: false });
  const [feeEstimates, setFeeEstimates] = useState<FeeEstimate>({ slow: 1, medium: 5, fast: 10 });
  const [utxos, setUtxos] = useState<UTXO[]>([]);
  const [sliderPercent, setSliderPercent] = useState<number>(100); // Key addition: slider for UTXO selection
  const [amountCurrency, setAmountCurrency] = useState<'BTC' | 'USD'>('BTC'); // Currency toggle
  const [btcPrice, setBtcPrice] = useState<number>(43000); // BTC/USD price

  // Computed values
  const sortedUtxos = [...utxos].sort((a, b) => {
    // Sort by ascending age/value as per planning document
    if (a.confirmations !== b.confirmations) {
      return a.confirmations - b.confirmations; // Older first (more confirmations)
    }
    return a.amount_sat - b.amount_sat; // Smaller amounts first
  });

  // Selected UTXOs based on slider percentage
  const selectedUtxosCount = Math.ceil((sortedUtxos.length * sliderPercent) / 100);
  const selectedUtxos = sortedUtxos.slice(0, selectedUtxosCount);
  
  // Calculate spendable balance from selected UTXOs only
  const spendableBalance = selectedUtxos.reduce((sum, utxo) => {
    // FIXED: Ensure amount_sat is treated as number and handle NaN values
    const sats = Number(utxo.amount_sat) || 0;
    return sum + sats;
  }, 0);

  // Load initial data
  useEffect(() => {
    loadInitialData();
  }, []);

  // Validate address when it changes
  useEffect(() => {
    if (recipientAddress) {
      const validation = sendService.validateBitcoinAddress(recipientAddress, 'mainnet');
      setAddressValidation(validation);
    } else {
      setAddressValidation({ valid: false });
    }
  }, [recipientAddress]);

  const loadInitialData = async () => {
    try {
      setLoading(true);
      setError(null);
      
      // Load fee estimates and BTC price
      const estimates = await sendService.getFeeEstimates();
      setFeeEstimates(estimates);

      // Load UTXOs (using dummy device ID for now)
      const fetchedUtxos = await sendService.getUtxos('keepkey-001', 0, 'p2wpkh');
      console.log('🔍 Fetched UTXOs:', fetchedUtxos.map(u => ({ txid: u.txid.substring(0, 8), vout: u.vout, amount_sat: u.amount_sat, type: typeof u.amount_sat })));
      setUtxos(fetchedUtxos);

      // Fetch BTC price (could be from same API as portfolio)
      try {
        const response = await fetch('http://localhost:1646/api/v2/portfolio/summary');
        if (response.ok) {
          // For now, use a reasonable default price - could be enhanced to get real price
          setBtcPrice(43000);
        }
      } catch (e) {
        console.log('Could not fetch BTC price, using default');
        setBtcPrice(43000);
      }
      
    } catch (error) {
      console.error('Error loading initial data:', error);
      setError('Failed to load wallet data');
    } finally {
      setLoading(false);
    }
  };

  const handleMaxAmount = () => {
    if (selectedUtxos.length === 0) return;
    
    // Calculate max using selected UTXOs minus estimated fee
    const totalSats = spendableBalance;
    const estimatedFeeSats = sendService.calculateFee(selectedUtxos, [{ amount: '0' }], feeEstimates[feeRate]);
    const maxAmountSats = totalSats - estimatedFeeSats;
    
    if (maxAmountSats > 0) {
      const maxBtc = sendService.satToBtc(maxAmountSats);
      setAmount(maxBtc);
    }
  };

  const handleSend = async () => {
    if (!recipientAddress || !amount || !addressValidation.valid) {
      setError('Please enter a valid recipient address and amount');
      return;
    }

    try {
      setLoading(true);
      setError(null);
      setSuccess(null);

      // Build transaction with slider percentage
      const buildRequest = {
        recipients: [{ address: recipientAddress, amount }],
        feeRate: feeEstimates[feeRate],
        sliderPercent: sliderPercent // Use actual slider value
      };

      const buildResult = await sendService.buildTransaction(buildRequest);
      
      if (!buildResult.success || !buildResult.tx) {
        throw new Error(buildResult.error || 'Failed to build transaction');
      }

      // Sign transaction
      const signResult = await sendService.signTransaction(
        buildResult.tx.psbt,
        'keepkey-001' // TODO: Get from device context
      );

      if (!signResult.success || !signResult.tx) {
        throw new Error(signResult.error || 'Failed to sign transaction');
      }

      // Broadcast transaction
      const broadcastResult = await sendService.broadcastTransaction(signResult.tx.psbt);

      if (!broadcastResult.success) {
        throw new Error(broadcastResult.error || 'Failed to broadcast transaction');
      }

      setSuccess(`Transaction sent successfully! TXID: ${broadcastResult.txid}`);
      
      // Reset form
      setRecipientAddress('');
      setAmount('');
      
      // Reload data
      await loadInitialData();

    } catch (error) {
      console.error('Error sending transaction:', error);
      setError(error instanceof Error ? error.message : 'Unknown error occurred');
    } finally {
      setLoading(false);
    }
  };

  const estimatedFeeSats = selectedUtxos.length > 0 && amount 
    ? sendService.calculateFee(selectedUtxos, [{ amount }], feeEstimates[feeRate])
    : 0;

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

        {/* Phase 1 MVP - Easy Mode Send Form */}
        <VStack gap={6} bg="gray.800" p={6} borderRadius="lg">
          {/* UTXO Selection - Choose which UTXOs to make available */}
          <Box w="100%">
            <Text color="gray.300" mb={2} fontSize="sm" fontWeight="medium">
              💰 Coin Selection ({sliderPercent}% of UTXOs)
            </Text>
            <HStack gap={2}>
              {[25, 50, 75, 100].map((percent) => (
                <Button
                  key={percent}
                  size="sm"
                  variant={sliderPercent === percent ? "solid" : "outline"}
                  colorScheme={sliderPercent === percent ? "green" : "gray"}
                  onClick={() => setSliderPercent(percent)}
                  flex="1"
                >
                  {percent}%
                </Button>
              ))}
            </HStack>
            <HStack justify="space-between" mt={1}>
              <Text fontSize="xs" color="gray.500">
                📊 Using {selectedUtxos.length} of {utxos.length} UTXOs
              </Text>
              <Text fontSize="xs" color="green.400" fontWeight="medium">
                💎 Available: {sendService.satToBtc(spendableBalance)} BTC
              </Text>
            </HStack>
          </Box>

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
                  alert('QR code scanning will be implemented in Phase 2');
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
                disabled={selectedUtxos.length === 0}
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
                const presetFeeRate = feeEstimates[preset];
                const feeSats = selectedUtxos.length > 0 ? sendService.calculateFee(selectedUtxos, [{ amount: amount || '0' }], presetFeeRate) : 0;
                const feeUsd = convertBtcToUsd(parseFloat(sendService.satToBtc(feeSats)));
                
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
                      <Text fontSize="2xs" opacity={0.6}>${feeUsd.toFixed(2)}</Text>
                    </VStack>
                  </Button>
                );
              })}
            </HStack>
            <Text fontSize="xs" color="gray.500" mt={2}>
              Estimated fee: {sendService.satToBtc(estimatedFeeSats)} BTC (${convertBtcToUsd(parseFloat(sendService.satToBtc(estimatedFeeSats))).toFixed(2)} USD)
            </Text>
          </Box>

          {/* Error Display */}
          {error && (
            <Box bg="red.900" p={3} borderRadius="md" border="1px solid" borderColor="red.600">
              <Text color="red.200" fontSize="sm">⚠️ {error}</Text>
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
            disabled={!addressValidation.valid || !amount || loading}
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