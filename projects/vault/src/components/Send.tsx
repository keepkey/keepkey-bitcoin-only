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
  // Simplified state for Phase 1 MVP
  const [recipientAddress, setRecipientAddress] = useState('');
  const [amount, setAmount] = useState('');
  const [feeRate, setFeeRate] = useState<'slow' | 'medium' | 'fast'>('medium');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [addressValidation, setAddressValidation] = useState<{ valid: boolean; error?: string }>({ valid: false });
  const [feeEstimates, setFeeEstimates] = useState<FeeEstimate>({ slow: 1, medium: 5, fast: 10 });
  const [utxos, setUtxos] = useState<UTXO[]>([]);
  const [spendableBalance, setSpendableBalance] = useState<number>(0);

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
      
      // Load fee estimates
      const estimates = await sendService.getFeeEstimates();
      setFeeEstimates(estimates);

      // Load UTXOs (using dummy device ID for now)
      const fetchedUtxos = await sendService.getUtxos('keepkey-001', 0, 'p2wpkh');
      setUtxos(fetchedUtxos);
      
      // Calculate spendable balance
      const totalBalance = fetchedUtxos.reduce((sum, utxo) => sum + utxo.amount_sat, 0);
      setSpendableBalance(totalBalance);
    } catch (error) {
      console.error('Error loading initial data:', error);
      setError('Failed to load wallet data');
    } finally {
      setLoading(false);
    }
  };

  const handleMaxAmount = () => {
    if (utxos.length === 0) return;
    
    // Simple calculation - use all UTXOs minus estimated fee
    const totalSats = utxos.reduce((sum, utxo) => sum + utxo.amount_sat, 0);
    const estimatedFeeSats = sendService.calculateFee(utxos, [{ amount }], feeEstimates[feeRate]);
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

      // Build transaction
      const buildRequest = {
        recipients: [{ address: recipientAddress, amount }],
        feeRate: feeEstimates[feeRate],
        sliderPercent: 100 // Use all available UTXOs for Phase 1
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

  const estimatedFeeSats = utxos.length > 0 && amount 
    ? sendService.calculateFee(utxos, [{ amount }], feeEstimates[feeRate])
    : 0;

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

          {/* Amount */}
          <Box w="100%">
            <HStack justify="space-between" mb={2}>
              <Text color="gray.300" fontSize="sm" fontWeight="medium">
                Amount (BTC)
              </Text>
              <Button
                size="xs"
                variant="outline"
                colorScheme="blue"
                onClick={handleMaxAmount}
                disabled={utxos.length === 0}
              >
                Max
              </Button>
            </HStack>
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
            <Text fontSize="xs" color="gray.500" mt={1}>
              Available: {sendService.satToBtc(spendableBalance)} BTC
            </Text>
          </Box>

          {/* Fee Selection */}
          <Box w="100%">
            <Text color="gray.300" mb={2} fontSize="sm" fontWeight="medium">
              Transaction Fee
            </Text>
            <HStack gap={2}>
              {(['slow', 'medium', 'fast'] as const).map((preset) => (
                <Button
                  key={preset}
                  size="sm"
                  variant={feeRate === preset ? "solid" : "outline"}
                  colorScheme={feeRate === preset ? "blue" : "gray"}
                  onClick={() => setFeeRate(preset)}
                  flex="1"
                  textTransform="capitalize"
                >
                  <VStack gap={1}>
                    <Text>{preset}</Text>
                    <Text fontSize="xs">{feeEstimates[preset]} sat/vB</Text>
                  </VStack>
                </Button>
              ))}
            </HStack>
            <Text fontSize="xs" color="gray.500" mt={2}>
              Estimated fee: {sendService.satToBtc(estimatedFeeSats)} BTC
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