import React, { useState, useEffect } from 'react';
import { 
  Box, 
  Heading, 
  Text, 
  VStack, 
  HStack, 
  Button, 
  IconButton,
  Flex,
  Spinner,
  Code,
  Badge,
  Input
} from '@chakra-ui/react';
import { FaArrowLeft, FaCopy, FaCheck, FaEye, FaChevronDown, FaChevronUp, FaEdit, FaSave, FaTimes, FaSync } from 'react-icons/fa';
import { SiBitcoin } from 'react-icons/si'; // Only if used in JSX
import QRCode from 'react-qr-code';
import { useWallet } from '../contexts/WalletContext';
import { useSettings } from '../contexts/SettingsContext';
import { useTypedTranslation } from '../hooks/useTypedTranslation';
import { PioneerAPI } from '../lib/api';

interface ReceiveProps {
  onBack?: () => void;
}

const Receive: React.FC<ReceiveProps> = ({ onBack }) => {
  const { portfolio, getReceiveAddress, selectAsset, loading: walletLoading, selectedAsset, fetchedXpubs } = useWallet();
  const { t } = useTypedTranslation('wallet');
  const { bitcoinAddressType } = useSettings();
  
  const [address, setAddress] = useState<string>('');
  const [hasCopied, setHasCopied] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [shouldGenerate, setShouldGenerate] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  
  // Map settings address type to local state format
  const getAddressTypeFromSettings = () => {
    switch (bitcoinAddressType) {
      case 'p2pkh':
        return 'legacy';
      case 'p2sh-p2wpkh':
        return 'segwit';
      case 'p2wpkh':
      default:
        return 'native-segwit';
    }
  };
  
  const getDerivationPathFromSettings = () => {
    switch (bitcoinAddressType) {
      case 'p2pkh':
        return "m/44'/0'/0'/0/0";
      case 'p2sh-p2wpkh':
        return "m/49'/0'/0'/0/0";
      case 'p2wpkh':
      default:
        return "m/84'/0'/0'/0/0";
    }
  };
  
  const [addressType, setAddressType] = useState<'legacy' | 'segwit' | 'native-segwit'>(getAddressTypeFromSettings());
  const [derivationPath, setDerivationPath] = useState<string>(getDerivationPathFromSettings());
  const [isTimeout, setIsTimeout] = useState(false);
  const [loadingStartTime, setLoadingStartTime] = useState<number | null>(null);
  
  // New states for address index and custom path editing
  const [currentAddressIndex, setCurrentAddressIndex] = useState<number>(0);
  const [newAddressCount, setNewAddressCount] = useState<number>(0);
  const [isEditingPath, setIsEditingPath] = useState<boolean>(false);
  const [customPath, setCustomPath] = useState<string>(getDerivationPathFromSettings());
  const [pathWarning, setPathWarning] = useState<string | null>(null);
  const [addressIndex, setAddressIndex] = useState<number>(0);
  const [maxUsedIndex, setMaxUsedIndex] = useState<number>(0);
  const [indexLoaded, setIndexLoaded] = useState<boolean>(false);
  const [changeIndex, setChangeIndex] = useState<number>(0);
  const [changeIndexLoaded, setChangeIndexLoaded] = useState<boolean>(false);

  // Function to refresh receive index from API
  const refreshReceiveIndex = async () => {
    if (!fetchedXpubs || fetchedXpubs.length === 0) {
      console.log('⚠️ No xpubs available for receive index refresh');
      return;
    }
    
    // 🐛 DEBUG: Show all available xpubs for debugging
    console.log('🔍 DEBUG: All available xpubs:', fetchedXpubs.map(x => ({
      path: x.path,
      type: x.xpub.startsWith('xpub') ? 'Legacy(P2PKH)' : 
            x.xpub.startsWith('ypub') ? 'SegWit(P2SH-P2WPKH)' :
            x.xpub.startsWith('zpub') ? 'Native SegWit(P2WPKH)' : 'Unknown',
      xpub: x.xpub
    })));
    
    try {
      const targetPath = bitcoinAddressType === 'p2pkh' ? "m/44'/0'/0'" :
                        bitcoinAddressType === 'p2sh-p2wpkh' ? "m/49'/0'/0'" :
                        "m/84'/0'/0'";
      
      const xpubData = fetchedXpubs.find(x => x.path === targetPath);
      if (xpubData) {
        console.log('🔄 Refreshing receive index for:', {
          addressType: bitcoinAddressType,
          targetPath,
          xpubType: xpubData.xpub.startsWith('xpub') ? 'Legacy(P2PKH)' : 
                    xpubData.xpub.startsWith('ypub') ? 'SegWit(P2SH-P2WPKH)' :
                    xpubData.xpub.startsWith('zpub') ? 'Native SegWit(P2WPKH)' : 'Unknown',
          fullXpub: xpubData.xpub
        });
        
        const response = await PioneerAPI.getReceiveAddress('Bitcoin', xpubData.xpub);
        console.log('📊 Receive index response:', response);
        
        const index = response?.receiveIndex ?? 0;
        if (response) {
          if (response.receiveIndex === null) {
            console.log('📋 No receive addresses used yet, starting at index 0');
            setAddressIndex(0);
            setMaxUsedIndex(0);
            setCurrentAddressIndex(0);
          } else if (typeof index === 'number' && index >= 0) {
            console.log('✅ Current receive index from API:', index);
            setAddressIndex(index);
            setMaxUsedIndex(index);
            setCurrentAddressIndex(index);
          }
          setIndexLoaded(true);
        }
      } else {
        console.log('⚠️ No xpub found for target path:', targetPath);
        console.log('📋 Available paths:', fetchedXpubs.map(x => x.path));
      }
    } catch (error) {
      console.error('❌ Failed to refresh receive index:', error);
      setIndexLoaded(true);
    }
  };
  
  // Function to refresh change index from API
  const refreshChangeIndex = async () => {
    if (!fetchedXpubs || fetchedXpubs.length === 0) {
      console.log('⚠️ No xpubs available for change index refresh');
      return;
    }
    
    try {
      const targetPath = bitcoinAddressType === 'p2pkh' ? "m/44'/0'/0'" :
                        bitcoinAddressType === 'p2sh-p2wpkh' ? "m/49'/0'/0'" :
                        "m/84'/0'/0'";
      
      const xpubData = fetchedXpubs.find(x => x.path === targetPath);
      if (xpubData) {
        console.log('🔄 Refreshing change index for:', {
          addressType: bitcoinAddressType,
          targetPath,
          xpubType: xpubData.xpub.startsWith('xpub') ? 'Legacy(P2PKH)' : 
                    xpubData.xpub.startsWith('ypub') ? 'SegWit(P2SH-P2WPKH)' :
                    xpubData.xpub.startsWith('zpub') ? 'Native SegWit(P2WPKH)' : 'Unknown',
          fullXpub: xpubData.xpub
        });
        
        // Use the PioneerAPI directly to get change address index
        const response = await PioneerAPI.getChangeAddress('Bitcoin', xpubData.xpub);
        console.log('📊 Change index response:', response);
        
        if (response && typeof response.changeIndex === 'number') {
          console.log('✅ Current change index from API:', response.changeIndex);
          setChangeIndex(response.changeIndex);
        } else {
          console.log('📋 No change index found, using 0');
          setChangeIndex(0);
        }
        setChangeIndexLoaded(true);
      }
    } catch (error) {
      console.error('❌ Failed to refresh change index:', error);
      setChangeIndex(0);
      setChangeIndexLoaded(true);
    }
  };

  // Component lifecycle debugging and fetch both receive and change indices
  useEffect(() => {
    console.log('🚀 Receive component mounted');
    
    // Fetch both indices when component mounts
    const fetchIndices = async () => {
      if (fetchedXpubs && fetchedXpubs.length > 0) {
        console.log('📊 Fetching both receive and change indices on component mount');
        
        // Fetch both indices in parallel
        await Promise.all([
          refreshReceiveIndex(),
          refreshChangeIndex()
        ]);
      }
    };
    
    fetchIndices();
    
    return () => {
      console.log('🛬 Receive component unmounting');
    };
  }, [fetchedXpubs, bitcoinAddressType]);

  // Get BTC asset from portfolio
  const btcAsset = portfolio?.assets.find(asset => asset.caip === 'bip122:000000000019d6689c085ae165831e93/slip44:0');

  // Debug portfolio and BTC asset
  useEffect(() => {
    console.log('🔍 Receive component - Portfolio:', portfolio);
    console.log('🔍 Receive component - All assets:', portfolio?.assets);
    console.log('🔍 Receive component - BTC Asset:', btcAsset);
    if (portfolio?.assets) {
      console.log('🔍 Receive component - Asset CAIPs:', portfolio.assets.map(a => ({ symbol: a.symbol, caip: a.caip })));
    }
  }, [portfolio, btcAsset]);

  // Update address type and path when settings change
  useEffect(() => {
    setAddressType(getAddressTypeFromSettings());
    const basePath = getDerivationPathFromSettings();
    // Update path with current address index - ensure addressIndex is a number
    const pathParts = basePath.split('/');
    const indexValue = typeof addressIndex === 'number' ? addressIndex : 0;
    pathParts[pathParts.length - 1] = indexValue.toString();
    const newPath = pathParts.join('/');
    setDerivationPath(newPath);
    setCustomPath(newPath);
  }, [bitcoinAddressType, addressIndex]);

  // Helper function to validate and parse custom path
  const validateCustomPath = (path: string): { valid: boolean; index?: number; warning?: string } => {
    const pathParts = path.split('/');
    if (pathParts.length !== 6) {
      return { valid: false, warning: 'Invalid path format' };
    }
    
    const index = parseInt(pathParts[5]);
    if (isNaN(index) || index < 0) {
      return { valid: false, warning: 'Invalid address index' };
    }
    
    // Check if index is too far from current max used index
    if (index > maxUsedIndex + 10) {
      return { 
        valid: true, 
        index, 
        warning: 'Warning: Address is more than 10 indices ahead. Funds sent here may not show in balances!' 
      };
    }
    
    return { valid: true, index };
  };

  // Generate receive address
  const generateAddress = (useCustomIndex?: number) => {
    console.log('🔍 generateAddress called with:', { useCustomIndex, addressIndex, addressIndexType: typeof addressIndex });
    
    if (!btcAsset) {
      setError(t('receive.noAssetFound'));
      return;
    }
    setLoading(true);
    setError(null);
    setAddress(''); // Clear any previous address before new generation
    setIsTimeout(false);
    setLoadingStartTime(Date.now());
    selectAsset(btcAsset);
    
    // Use custom index if provided, otherwise use current addressIndex
    // Ensure we have a valid number
    let indexToUse: number;
    if (typeof useCustomIndex === 'number') {
      indexToUse = useCustomIndex;
    } else if (typeof addressIndex === 'number') {
      indexToUse = addressIndex;
    } else {
      console.warn('⚠️ addressIndex is not a number:', addressIndex, 'defaulting to 0');
      indexToUse = 0; // Default to 0 if neither is valid
    }
    
    console.log('📍 Using index:', indexToUse);
    
    // Use address type and update derivation path with index
    setAddressType(getAddressTypeFromSettings());
    const basePath = getDerivationPathFromSettings();
    const pathParts = basePath.split('/');
    pathParts[pathParts.length - 1] = indexToUse.toString();
    const newPath = pathParts.join('/');
    setDerivationPath(newPath);
    
    // Update current index
    setCurrentAddressIndex(indexToUse);
    
    setShouldGenerate(true); // Triggers useEffect below
  };

  useEffect(() => {
    if (shouldGenerate && btcAsset) {
      (async () => {
        try {
          console.log('🎯 Starting address generation...');
          console.log('🔐 Using Bitcoin address type:', bitcoinAddressType);
          console.log('📍 Using derivation path:', derivationPath);
          
          // Pass the custom derivation path to getReceiveAddress
          const addr = await getReceiveAddress(bitcoinAddressType, derivationPath);
          console.log('📬 Received address from getReceiveAddress:', addr);
          if (addr) {
            setAddress(addr);
            setError(null);
            console.log('✅ Address set in state:', addr);
          } else {
            console.error('❌ No address returned from getReceiveAddress');
            setError(t('receive.failedNoResponse'));
          }
        } catch (e) {
          console.error('❌ Error in address generation:', e);
          setError(
            e instanceof Error ? e.message : t('receive.failedEnsureConnected')
          );
        } finally {
          setLoading(false);
          setShouldGenerate(false);
        }
      })();
    }
  }, [shouldGenerate, btcAsset, getReceiveAddress, derivationPath]);

  // Copy address to clipboard
  const onCopy = () => {
    if (address) {
      navigator.clipboard.writeText(address);
      setHasCopied(true);
      setTimeout(() => setHasCopied(false), 2000);
    }
  };
  
  // Handle getting a new address
  const handleGetNewAddress = async () => {
    if (newAddressCount >= 3) {
      setError('Maximum of 3 new addresses per session reached');
      return;
    }
    
    // Get fresh address from Pioneer API instead of just incrementing
    try {
      const targetPath = bitcoinAddressType === 'p2pkh' ? "m/44'/0'/0'" :
                        bitcoinAddressType === 'p2sh-p2wpkh' ? "m/49'/0'/0'" :
                        "m/84'/0'/0'";
      
      const xpubData = fetchedXpubs.find(x => x.path === targetPath);
      if (xpubData) {
        console.log('🔄 Getting new address from Pioneer API...');
        const response = await PioneerAPI.getReceiveAddress('Bitcoin', xpubData.xpub);
        
        if (response && typeof response.addressIndex === 'number') {
          console.log('✅ New address index from API:', response.addressIndex);
          setAddressIndex(response.addressIndex);
          setMaxUsedIndex(Math.max(maxUsedIndex, response.addressIndex));
          setCurrentAddressIndex(response.addressIndex);
          setNewAddressCount(prev => prev + 1);
          setAddress(''); // Clear current address
          generateAddress(response.addressIndex);
        } else {
          // Fallback to incrementing
          const newIndex = addressIndex + 1;
          setAddressIndex(newIndex);
          setNewAddressCount(prev => prev + 1);
          setAddress(''); // Clear current address
          generateAddress(newIndex);
        }
      } else {
        // Fallback if no xpub
        const newIndex = addressIndex + 1;
        setAddressIndex(newIndex);
        setNewAddressCount(prev => prev + 1);
        setAddress(''); // Clear current address
        generateAddress(newIndex);
      }
    } catch (error) {
      console.error('❌ Failed to get new address from API:', error);
      // Fallback to incrementing on error
      const newIndex = addressIndex + 1;
      setAddressIndex(newIndex);
      setNewAddressCount(prev => prev + 1);
      setAddress(''); // Clear current address
      generateAddress(newIndex);
    }
  };
  
  // Handle custom path editing
  const handleSaveCustomPath = () => {
    const validation = validateCustomPath(customPath);
    if (!validation.valid) {
      setError(validation.warning || 'Invalid path');
      return;
    }
    
    if (validation.warning) {
      setPathWarning(validation.warning);
    } else {
      setPathWarning(null);
    }
    
    if (validation.index !== undefined) {
      setAddressIndex(validation.index);
      setCurrentAddressIndex(validation.index);
      setAddress(''); // Clear current address
      generateAddress(validation.index);
    }
    
    setIsEditingPath(false);
  };
  
  // Handle custom path input change
  const handlePathInputChange = (value: string) => {
    // Only allow editing of numbers, not slashes
    const pathParts = value.split('/');
    const currentParts = customPath.split('/');
    
    // Ensure we have the same number of parts
    if (pathParts.length !== currentParts.length) {
      return;
    }
    
    // Only update if the structure is maintained (slashes in same positions)
    let valid = true;
    for (let i = 0; i < pathParts.length - 1; i++) {
      // Check all parts except the last one remain unchanged
      if (i < pathParts.length - 2 && pathParts[i] !== currentParts[i]) {
        valid = false;
        break;
      }
    }
    
    if (valid) {
      setCustomPath(value);
    }
  };

  // Monitor address state changes
  useEffect(() => {
    console.log('📍 Address state changed:', address);
  }, [address]);

  // Timeout mechanism to show retry option
  useEffect(() => {
    if (!loading || !loadingStartTime) return;

    const timeoutId = setTimeout(() => {
      if (loading && !address) {
        setIsTimeout(true);
      }
    }, 15000); // Show timeout message after 15 seconds

    return () => clearTimeout(timeoutId);
  }, [loading, loadingStartTime, address]);

  // Log render branch
  useEffect(() => {
    if (loading) {
      console.log('[RENDER] Receive: loading spinner branch');
    } else if (!address) {
      console.log('[RENDER] Receive: no address branch');
    } else {
      console.log('[RENDER] Receive: address display branch, address:', address);
    }
  }, [loading, address]);

  // Get relevant xpub for the current address type
  const getRelevantXpub = () => {
    if (!fetchedXpubs || fetchedXpubs.length === 0) return null;
    
    // Map address type to derivation path prefix
    const pathPrefix = addressType === 'legacy' ? "m/44'/0'/0'" :
                      addressType === 'segwit' ? "m/49'/0'/0'" :
                      "m/84'/0'/0'"; // native-segwit
    
    return fetchedXpubs.find(xpub => xpub.path === pathPrefix);
  };

  const relevantXpub = getRelevantXpub();

  if (walletLoading) {
    return (
      <Box height="100%" display="flex" alignItems="center" justifyContent="center" bg="transparent">
        <VStack gap={4}>
          <Spinner size="xl" color="blue.400" />
          <Text color="gray.300" fontSize="lg">{t('receive.loadingWallet')}</Text>
        </VStack>
      </Box>
    );
  }

  return (
    <Box height="100%" bg="transparent" display="flex" alignItems="center" justifyContent="center" p={4}>
      <Box 
        maxW={showAdvanced ? "1200px" : "900px"}
        w="100%"
        bg="rgba(26, 32, 44, 0.95)" 
        p={8} 
        borderRadius="xl" 
        backdropFilter="blur(20px)"
        border="1px solid rgba(255, 255, 255, 0.1)"
        boxShadow="2xl"
        transition="all 0.3s ease"
      >
        <VStack align="stretch" gap={6}>
          {/* Header */}
          <HStack>
            <IconButton
              aria-label={t('receive.goBack')}
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
                {t('receive.titleBitcoin')}
              </Heading>
            </Flex>
            {/* Edit path button in top right */}
            {address && (
              <IconButton
                aria-label="Edit path"
                onClick={() => setIsEditingPath(!isEditingPath)}
                size="sm"
                variant="ghost"
                color="gray.400"
                _hover={{ color: "white", bg: "gray.700" }}
              >
                {isEditingPath ? <FaTimes /> : <FaEdit />}
              </IconButton>
            )}
          </HStack>

          {/* Main Content */}
          <VStack gap={6} bg="gray.800" p={8} borderRadius="lg" minH="400px" justify="center">
            {loading ? (
              /* Show spinner and message while loading, regardless of address state */
              <VStack gap={4} textAlign="center">
                <Spinner size="xl" color="blue.400" />
                <Text color="gray.300" fontSize="lg">{t('receive.generatingAddress')}</Text>
                <Text color="gray.400" fontSize="sm">
                  {t('receive.generatingAddressHint')}
                </Text>
                
                {isTimeout && (
                  <VStack gap={3} mt={4}>
                    <Box bg="yellow.900" p={3} borderRadius="md" border="1px solid" borderColor="yellow.600">
                      <Text color="yellow.200" fontSize="sm">
                        {t('receive.timeout.icon')} {t('receive.timeout.message')}
                      </Text>
                    </Box>
                    <Button
                      colorScheme="blue"
                      variant="outline"
                      size="md"
                      onClick={() => {
                        setLoading(false);
                        setIsTimeout(false);
                        setLoadingStartTime(null);
                        setTimeout(() => generateAddress(), 100);
                      }}
                    >
                      {t('receive.retry')}
                    </Button>
                  </VStack>
                )}
                
                {error && (
                  <Box bg="red.900" p={3} borderRadius="md" border="1px solid" borderColor="red.600">
                    <Text color="red.200" fontSize="sm">{t('receive.error.icon')} {error}</Text>
                  </Box>
                )}
              </VStack>
            ) : !address ? (
              /* Generate Address View */
              <VStack gap={4} textAlign="center">
                <Box color="orange.400" fontSize="4xl">
                  <SiBitcoin />
                </Box>
                <Text color="gray.300" fontSize="lg" fontWeight="medium">
                  {t('receive.generateReceiveAddress')}
                </Text>
                <Text color="gray.400" fontSize="sm" textAlign="center" maxW="300px">
                  {t('receive.generateAddressDescription')}
                </Text>
                
                {/* Index Display on Main Page */}
                <VStack gap={3} bg="gray.900" p={4} borderRadius="md" border="1px solid" borderColor="gray.700">
                  <Text color="gray.300" fontSize="sm" fontWeight="medium">Address Indices</Text>
                  <HStack gap={6} justify="center">
                    <VStack gap={1}>
                      <Text color="gray.400" fontSize="xs">Receive Index</Text>
                      <Button
                        size="sm"
                        variant="outline"
                        colorScheme="blue"
                        onClick={refreshReceiveIndex}
                        loading={!indexLoaded}
                        loadingText="..."
                        minW="60px"
                        title="Click to refresh receive index"
                      >
                        {indexLoaded ? addressIndex : '...'}
                      </Button>
                    </VStack>
                    <VStack gap={1}>
                      <Text color="gray.400" fontSize="xs">Change Index</Text>
                      <Button
                        size="sm"
                        variant="outline"
                        colorScheme="green"
                        onClick={refreshChangeIndex}
                        loading={!changeIndexLoaded}
                        loadingText="..."
                        minW="60px"
                        title="Click to refresh change index"
                      >
                        {changeIndexLoaded ? changeIndex : '...'}
                      </Button>
                    </VStack>
                  </HStack>
                  <Text color="gray.500" fontSize="xs" textAlign="center">
                    Click numbers to refresh from Pioneer API
                  </Text>
                </VStack>
                
                {error && (
                  <Box bg="red.900" p={3} borderRadius="md" border="1px solid" borderColor="red.600">
                    <Text color="red.200" fontSize="sm">{t('receive.error.icon')} {error}</Text>
                  </Box>
                )}
                <Button
                  colorScheme="blue"
                  size="lg"
                  onClick={() => generateAddress()}
                  disabled={loading || !btcAsset}
                  minW="200px"
                >
                  <HStack gap={2}>
                    <FaEye />
                    <Text>{t('receive.generateAddress')}</Text>
                  </HStack>
                </Button>
                {!btcAsset && (
                  <Text color="yellow.400" fontSize="xs" textAlign="center">
                    {t('receive.noBitcoinAsset')}
                  </Text>
                )}
              </VStack>
            ) : (
              /* Address Display View */
              <VStack gap={6} w="100%">
                {/* Success Header */}
                <VStack gap={2} textAlign="center">
                  <Text color="green.300" fontSize="lg" fontWeight="bold">
                    {t('receive.success.icon')} {t('receive.addressGenerated')}
                  </Text>
                  <Text color="gray.400" fontSize="sm">
                    {t('receive.useThisAddress')}
                  </Text>
                </VStack>

                {/* Main Address Display - Horizontal Layout */}
                <HStack gap={8} w="100%" align="flex-start" justify="center">
                  {/* QR Code */}
                  <Box
                    w="240px"
                    h="240px"
                    bg="white"
                    borderRadius="lg"
                    display="flex"
                    alignItems="center"
                    justifyContent="center"
                    p={3}
                    flexShrink={0}
                  >
                    <QRCode
                      value={address}
                      size={216}
                      style={{ height: "auto", maxWidth: "100%", width: "100%" }}
                      viewBox={`0 0 216 216`}
                    />
                  </Box>

                  {/* Address Info */}
                  <VStack flex="1" gap={4} align="stretch" justify="center" minH="240px">
                    <VStack gap={2} align="stretch">
                      <Text color="gray.300" fontSize="md" fontWeight="medium">
                        {t('receive.bitcoinAddress')}
                      </Text>
                      <Box
                        bg="gray.700"
                        p={4}
                        borderRadius="md"
                        border="1px solid"
                        borderColor="gray.600"
                      >
                        <Text
                          color="white"
                          fontSize="md"
                          fontFamily="mono"
                          wordBreak="break-all"
                          lineHeight="1.5"
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
                          <Text>{hasCopied ? t('receive.addressCopied') : t('receive.copyAddress')}</Text>
                        </HStack>
                      </Button>
                      
                      <Button
                        variant="outline"
                        colorScheme="gray"
                        size="lg"
                        onClick={handleGetNewAddress}
                        flex="1"
                        disabled={newAddressCount >= 3}
                      >
                        <HStack gap={2}>
                          <FaSync />
                          <Text>
                            {newAddressCount >= 3 
                              ? 'Max reached'
                              : `New Address (${3 - newAddressCount} left)`}
                          </Text>
                        </HStack>
                      </Button>
                    </HStack>

                    {/* Advanced View Toggle */}
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setShowAdvanced(!showAdvanced)}
                      color="gray.400"
                      _hover={{ color: "gray.200" }}
                      alignSelf="center"
                    >
                      <HStack gap={1}>
                        <Text fontSize="xs">{t('receive.advancedView')}</Text>
                        {showAdvanced ? <FaChevronUp /> : <FaChevronDown />}
                      </HStack>
                    </Button>
                  </VStack>

                  {/* Advanced View Content - Third Column */}
                  {showAdvanced && (
                    <VStack
                      w="320px"
                      gap={4}
                      bg="gray.900"
                      p={4}
                      borderRadius="md"
                      border="1px solid"
                      borderColor="gray.700"
                      align="stretch"
                      flexShrink={0}
                      maxH="400px"
                      overflowY="auto"
                    >
                      <Text color="gray.300" fontSize="sm" fontWeight="medium" textAlign="center">
                        {t('receive.technicalDetails')}
                      </Text>
                      
                      <VStack align="stretch" gap={3}>
                        {/* Address Type */}
                        <HStack justify="space-between">
                          <Text color="gray.400" fontSize="xs">{t('receive.addressType.label')}:</Text>
                          <Badge colorScheme="blue" fontSize="xs">
                            {addressType === 'legacy' ? t('receive.addressType.legacy') :
                             addressType === 'segwit' ? t('receive.addressType.segwit') :
                             t('receive.addressType.nativeSegwit')}
                          </Badge>
                        </HStack>

                        <Box w="100%" h="1px" bg="gray.700" />

                        {/* Derivation Path with editing */}
                        <VStack align="stretch" gap={1}>
                          <HStack justify="space-between">
                            <Text color="gray.400" fontSize="xs">{t('receive.derivationPath')}:</Text>
                            {isEditingPath && (
                              <HStack gap={1}>
                                <IconButton
                                  size="xs"
                                  aria-label="Save"
                                  onClick={handleSaveCustomPath}
                                  colorScheme="green"
                                  variant="ghost"
                                >
                                  <FaSave />
                                </IconButton>
                                <IconButton
                                  size="xs"
                                  aria-label="Cancel"
                                  onClick={() => {
                                    setIsEditingPath(false);
                                    setCustomPath(derivationPath);
                                    setPathWarning(null);
                                  }}
                                  colorScheme="red"
                                  variant="ghost"
                                >
                                  <FaTimes />
                                </IconButton>
                              </HStack>
                            )}
                          </HStack>
                          {isEditingPath ? (
                            <Box>
                              <Input
                                value={customPath}
                                onChange={(e) => handlePathInputChange(e.target.value)}
                                bg="gray.800"
                                color="blue.300"
                                borderColor={pathWarning ? "red.600" : "gray.600"}
                                fontSize="xs"
                                fontFamily="mono"
                                size="sm"
                                px={2}
                                py={1}
                                _hover={{
                                  borderColor: pathWarning ? "red.500" : "gray.500"
                                }}
                                _focus={{
                                  borderColor: pathWarning ? "red.400" : "blue.400",
                                  boxShadow: "none"
                                }}
                              />
                              {pathWarning && (
                                <Text color="red.400" fontSize="xs" mt={1}>
                                  ⚠️ {pathWarning}
                                </Text>
                              )}
                            </Box>
                          ) : (
                            <Code
                              bg="gray.800"
                              color="blue.300"
                              p={2}
                              borderRadius="md"
                              fontSize="xs"
                              border={pathWarning ? '1px solid' : 'none'}
                              borderColor={pathWarning ? 'red.600' : undefined}
                            >
                              {derivationPath}
                            </Code>
                          )}
                        </VStack>

                        <Box w="100%" h="1px" bg="gray.700" />

                        {/* Current Address Index */}
                        <HStack justify="space-between">
                          <Text color="gray.400" fontSize="xs">Current Index:</Text>
                          <Badge colorScheme="purple" fontSize="xs">
                            {currentAddressIndex}
                          </Badge>
                        </HStack>

                        <Box w="100%" h="1px" bg="gray.700" />

                        {/* Extended Public Key (if available) */}
                        {relevantXpub && (
                          <>
                            <VStack align="stretch" gap={1}>
                              <HStack justify="space-between" align="center">
                                <Text color="gray.400" fontSize="xs">{t('receive.accountExtendedPublicKey')}:</Text>
                                <Button
                                  size="xs"
                                  variant="ghost"
                                  colorScheme="blue"
                                  onClick={() => {
                                    navigator.clipboard.writeText(relevantXpub.xpub);
                                    // Could add a toast notification here
                                  }}
                                  title="Copy to clipboard"
                                >
                                  {t('receive.copy')}
                                </Button>
                              </HStack>
                              <Code
                                bg="gray.800"
                                color="gray.300"
                                p={2}
                                borderRadius="md"
                                fontSize="xs"
                                wordBreak="break-all"
                                whiteSpace="pre-wrap"
                              >
                                {relevantXpub.xpub}
                              </Code>
                              <Text color="gray.500" fontSize="xs" mt={1}>
                                {t('receive.path')}: {relevantXpub.path}
                              </Text>
                            </VStack>
                            <Box w="100%" h="1px" bg="gray.700" />
                          </>
                        )}

                        {/* Script Type Info */}
                        <VStack align="stretch" gap={1}>
                          <Text color="gray.400" fontSize="xs">{t('receive.scriptType')}:</Text>
                          <Text color="gray.300" fontSize="xs">
                            {addressType === 'legacy' ? t('receive.scriptTypeInfo.legacy') :
                             addressType === 'segwit' ? t('receive.scriptTypeInfo.segwit') :
                             t('receive.scriptTypeInfo.nativeSegwit')}
                          </Text>
                        </VStack>

                        <Box w="100%" h="1px" bg="gray.700" />

                        {/* Address Format Info */}
                        <HStack justify="space-between">
                          <Text color="gray.400" fontSize="xs">{t('receive.addressFormat')}:</Text>
                          <Text color="gray.300" fontSize="xs">
                            {addressType === 'legacy' ? t('receive.addressFormatInfo.legacy') :
                             addressType === 'segwit' ? t('receive.addressFormatInfo.segwit') :
                             t('receive.addressFormatInfo.nativeSegwit')}
                          </Text>
                        </HStack>

                        {/* Device Info */}
                        <Box w="100%" h="1px" bg="gray.700" />
                        <VStack align="stretch" gap={1}>
                          <Text color="gray.400" fontSize="xs">{t('receive.generatedBy')}:</Text>
                          <HStack>
                            <Badge colorScheme="green" fontSize="xs">{t('receive.keepKeyDevice')}</Badge>
                            <Text color="gray.500" fontSize="xs">{t('receive.hardwareWallet')}</Text>
                          </HStack>
                        </VStack>
                      </VStack>

                      {/* Info Note */}
                      <Box bg="blue.900" p={3} borderRadius="md" border="1px solid" borderColor="blue.700">
                        <Text color="blue.200" fontSize="xs">
                          {t('receive.info.icon')} {t('receive.info.addressDerivation')}
                        </Text>
                      </Box>
                    </VStack>
                  )}
                </HStack>

                {/* Security Notice */}
                <Box bg="blue.900" p={3} borderRadius="md" border="1px solid" borderColor="blue.600">
                  <Text color="blue.200" fontSize="xs" textAlign="center">
                    {t('receive.security.icon')} {t('receive.security.verifyAddress')}
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
            {t('receive.backToVault')}
          </Button>
        </VStack>
      </Box>
    </Box>
  );
};

export default Receive; 