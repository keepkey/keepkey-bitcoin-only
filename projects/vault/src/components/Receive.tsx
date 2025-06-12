import React, { useState } from 'react';
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
import { FaArrowLeft, FaCopy, FaCheck, FaEye, FaRedo, FaPencilAlt } from 'react-icons/fa';
import { FiShield } from 'react-icons/fi';
import { SiBitcoin } from 'react-icons/si';
import QRCode from 'react-qr-code';

interface ReceiveProps {
  onBack?: () => void;
}

// Address state
interface AddressState {
  address: string;
  addressIndex: number;
  derivationPath: string;
  scriptType: string;
  isChange: boolean;
  account: number;
}

// Wallet State following planning document
interface WalletState {
  selectedAccount: number;
  selectedScriptType: 'p2pkh' | 'p2sh-p2wpkh' | 'p2wpkh';
  showChangeAddresses: boolean;
  currentAddressIndex: number;
  isOnlineMode: boolean;
}

const Receive: React.FC<ReceiveProps> = ({ onBack }) => {
  // State management following planning document
  const [walletState, setWalletState] = useState<WalletState>({
    selectedAccount: 0,
    selectedScriptType: 'p2wpkh', // Default: Native SegWit
    showChangeAddresses: false,
    currentAddressIndex: 0,
    isOnlineMode: true
  });
  
  const [addressState, setAddressState] = useState<AddressState | null>(null);
  const [hasCopied, setHasCopied] = useState(false);
  const [deviceVerified, setDeviceVerified] = useState(false);
  const [verifying, setVerifying] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editingPath, setEditingPath] = useState(false);
  const [customPath, setCustomPath] = useState('');

  // Script type info mapping
  const getScriptTypeInfo = (scriptType: string) => {
    switch (scriptType) {
      case 'p2pkh':
        return { bip: 44, name: "Legacy (P2PKH)", pathPrefix: "m/44'/0'" };
      case 'p2sh-p2wpkh':
        return { bip: 49, name: "SegWit (P2SH-P2WPKH)", pathPrefix: "m/49'/0'" };
      case 'p2wpkh':
        return { bip: 84, name: "Native SegWit (P2WPKH)", pathPrefix: "m/84'/0'" };
      default:
        return { bip: 84, name: "Native SegWit (P2WPKH)", pathPrefix: "m/84'/0'" };
    }
  };

  // Create derivation path array
  const createDerivationPath = (scriptType: string, account: number, isChange: boolean, index: number): number[] => {
    const { bip } = getScriptTypeInfo(scriptType);
    const hardened = 0x80000000;
    return [
      hardened + bip,     // BIP number'
      hardened + 0,       // Bitcoin'
      hardened + account, // Account'
      isChange ? 1 : 0,   // Change chain
      index               // Address index
    ];
  };

  // Format derivation path for display
  const formatDerivationPath = (scriptType: string, account: number, isChange: boolean, index: number) => {
    const { pathPrefix } = getScriptTypeInfo(scriptType);
    const changeStr = isChange ? '1' : '0';
    return `${pathPrefix}/${account}'/${changeStr}/${index}`;
  };

  // FORCE DEVICE VERIFICATION FIRST
  const verifyOnDevice = async () => {
    setVerifying(true);
    setError(null);
    
    try {
      console.log("🔐 FORCING device verification before showing address...");
      
      const derivationPath = createDerivationPath(
        walletState.selectedScriptType, 
        walletState.selectedAccount, 
        walletState.showChangeAddresses, 
        walletState.currentAddressIndex
      );
      
      console.log("Derivation path:", derivationPath);
      console.log("Using index:", walletState.currentAddressIndex);
      
      const requestBody = {
        coin: 'Bitcoin',
        address_n: derivationPath,
        script_type: walletState.selectedScriptType,
        show_display: true // ALWAYS force device display first
      };
      
      console.log("Request body:", JSON.stringify(requestBody, null, 2));
      
      const response = await fetch('http://localhost:1646/addresses/utxo', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Bearer 1fa0c776-eaa9-499d-a2e5-f76af6073912'
        },
        body: JSON.stringify(requestBody)
      });

      if (response.ok) {
        const data = await response.json();
        console.log("✅ Device verification successful, address:", data.address);
        
        // Set address state with verified address
        setAddressState({
          address: data.address,
          addressIndex: walletState.currentAddressIndex,
          derivationPath: formatDerivationPath(
            walletState.selectedScriptType,
            walletState.selectedAccount, 
            walletState.showChangeAddresses, 
            walletState.currentAddressIndex
          ),
          scriptType: walletState.selectedScriptType,
          isChange: walletState.showChangeAddresses,
          account: walletState.selectedAccount
        });
        
        setDeviceVerified(true);
        alert("✅ Address verified on device! You can now safely use this address.");
      } else {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
    } catch (error) {
      console.error('Device verification failed:', error);
      setError('Device verification failed. Please ensure your KeepKey is connected and try again.');
    } finally {
      setVerifying(false);
    }
  };

  // Generate new address (increment index)
  const generateNewAddress = async () => {
    if (!deviceVerified) {
      setError('Please verify on device first!');
      return;
    }
    
    const newIndex = walletState.currentAddressIndex + 1;
    setWalletState(prev => ({ ...prev, currentAddressIndex: newIndex }));
    setDeviceVerified(false); // Force re-verification for new address
    setAddressState(null);
    setError('New address generated. Please verify on device before using.');
  };

  // Copy address to clipboard
  const onCopy = () => {
    if (addressState?.address) {
      navigator.clipboard.writeText(addressState.address);
      setHasCopied(true);
      setTimeout(() => setHasCopied(false), 2000);
    }
  };

  // Handle settings changes
  const handleScriptTypeChange = (value: string) => {
    setWalletState(prev => ({ ...prev, selectedScriptType: value as any }));
    setDeviceVerified(false);
    setAddressState(null);
  };

  const handleAccountChange = (value: number) => {
    setWalletState(prev => ({ ...prev, selectedAccount: value }));
    setDeviceVerified(false);
    setAddressState(null);
  };

  const handleChangeToggle = (checked: boolean) => {
    setWalletState(prev => ({ ...prev, showChangeAddresses: checked }));
    setDeviceVerified(false);
    setAddressState(null);
  };

  const handleIndexChange = (value: number) => {
    setWalletState(prev => ({ ...prev, currentAddressIndex: value }));
    setDeviceVerified(false);
    setAddressState(null);
  };

  // Reset to fresh state
  const resetToFresh = () => {
    setDeviceVerified(false);
    setAddressState(null);
    setError(null);
    setWalletState(prev => ({ ...prev, currentAddressIndex: 0 }));
  };

  return (
    <Box height="100%" bg="transparent" display="flex" alignItems="center" justifyContent="center" p={6}>
      <Box 
        maxW="900px" 
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
          <Box w="40px" />
        </HStack>

        {/* Main Content - Side by Side Layout */}
        <HStack gap={6} align="stretch" minH="500px">
          {/* Left Panel - Configuration */}
          <VStack gap={4} flex="1" align="stretch" maxW="50%">
            {/* Configuration Controls */}
            <VStack gap={4} bg="gray.800" p={4} borderRadius="lg" h="500px" justify="space-between">
              {/* Top Section - Configuration Controls */}
              <VStack gap={4} align="stretch" w="100%">
                <Text color="gray.300" fontSize="sm" fontWeight="bold">
                  Address Configuration
                </Text>
                
                {/* Account Dropdown */}
                <VStack align="stretch" gap={2}>
                  <Text color="gray.400" fontSize="xs">Account</Text>
                  <select
                    value={walletState.selectedAccount}
                    onChange={(e) => handleAccountChange(parseInt(e.target.value))}
                    style={{ 
                      backgroundColor: '#2D3748', 
                      borderColor: '#4A5568', 
                      color: 'white',
                      border: '1px solid #4A5568',
                      borderRadius: '6px',
                      padding: '8px',
                      fontSize: '14px',
                      width: '100%'
                    }}
                  >
                    <option value={0}>Account 1</option>
                    <option value={1}>Account 2</option>
                    <option value={2}>Account 3</option>
                  </select>
                </VStack>

                {/* Script Type Dropdown */}
                <VStack align="stretch" gap={2}>
                  <Text color="gray.400" fontSize="xs">Address Type</Text>
                  <select
                    value={walletState.selectedScriptType}
                    onChange={(e) => handleScriptTypeChange(e.target.value)}
                    style={{ 
                      backgroundColor: '#2D3748', 
                      borderColor: '#4A5568', 
                      color: 'white',
                      border: '1px solid #4A5568',
                      borderRadius: '6px',
                      padding: '8px',
                      fontSize: '14px',
                      width: '100%'
                    }}
                  >
                    <option value="p2wpkh">Native SegWit (P2WPKH) - Recommended</option>
                    <option value="p2sh-p2wpkh">SegWit (P2SH-P2WPKH)</option>
                    <option value="p2pkh">Legacy (P2PKH)</option>
                  </select>
                </VStack>

                {/* Change Address Toggle */}
                <HStack justify="space-between" align="center">
                  <Text color="gray.400" fontSize="xs">Change Addresses</Text>
                  <label style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <input
                      type="checkbox"
                      checked={walletState.showChangeAddresses}
                      onChange={(e) => handleChangeToggle(e.target.checked)}
                      style={{ accentColor: '#3182ce' }}
                    />
                    <Text color="gray.400" fontSize="xs">
                      {walletState.showChangeAddresses ? 'ON' : 'OFF'}
                    </Text>
                  </label>
                </HStack>

                {/* Current Path Display with Edit */}
                <VStack align="stretch" gap={2}>
                  <HStack justify="space-between" align="center">
                    <Text color="gray.400" fontSize="xs">Derivation Path</Text>
                    <IconButton
                      aria-label="Edit path manually"
                      size="xs"
                      variant="ghost"
                      color="gray.300"
                      onClick={() => setEditingPath(true)}
                    >
                      <FaPencilAlt />
                    </IconButton>
                  </HStack>
                  {editingPath ? (
                    <HStack>
                      <Input
                        value={customPath || formatDerivationPath(
                          walletState.selectedScriptType,
                          walletState.selectedAccount, 
                          walletState.showChangeAddresses, 
                          walletState.currentAddressIndex
                        )}
                        onChange={(e) => setCustomPath(e.target.value)}
                        bg="gray.700"
                        borderColor="gray.600"
                        color="white"
                        size="sm"
                        fontSize="xs"
                        fontFamily="mono"
                        placeholder="m/84'/0'/0'/0/0"
                      />
                      <Button size="xs" colorScheme="green" onClick={() => setEditingPath(false)}>
                        ✓
                      </Button>
                      <Button size="xs" variant="ghost" onClick={() => { setEditingPath(false); setCustomPath(''); }}>
                        ✗
                      </Button>
                    </HStack>
                  ) : (
                    <Text color="gray.300" fontSize="xs" fontFamily="mono" bg="gray.700" p={2} borderRadius="md">
                      {formatDerivationPath(
                        walletState.selectedScriptType,
                        walletState.selectedAccount, 
                        walletState.showChangeAddresses, 
                        walletState.currentAddressIndex
                      )} (Index: {walletState.currentAddressIndex})
                    </Text>
                  )}
                </VStack>
              </VStack>

              {/* Bottom Section - Action Buttons */}
              <VStack gap={3} w="100%">
                {deviceVerified ? (
                  <>
                    <HStack gap={3} width="100%">
                      <Button
                        colorScheme="gray"
                        size="lg"
                        onClick={generateNewAddress}
                        flex="1"
                        bg="gray.600"
                        _hover={{ bg: "gray.500" }}
                        color="white"
                      >
                        <HStack gap={2}>
                          <FaRedo />
                          <Text>Generate Next</Text>
                        </HStack>
                      </Button>
                      <Button
                        colorScheme="gray"
                        size="lg"
                        onClick={verifyOnDevice}
                        disabled={verifying}
                        flex="1"
                        bg="gray.600"
                        _hover={{ bg: "gray.500" }}
                        color="white"
                      >
                        <HStack gap={2}>
                          {verifying ? <Spinner size="sm" /> : <FaEye />}
                          <Text>Re-verify</Text>
                        </HStack>
                      </Button>
                    </HStack>
                    
                    <Button
                      variant="ghost"
                      color="gray.400"
                      onClick={resetToFresh}
                      size="sm"
                    >
                      Reset to Fresh
                    </Button>
                  </>
                ) : (
                  /* Spacer when not verified to maintain height */
                  <Box minH="90px" />
                )}
              </VStack>
            </VStack>

            {/* Error Display */}
            {error && (
              <Box bg="red.900" p={3} borderRadius="md" border="1px solid" borderColor="red.600">
                <Text color="red.200" fontSize="sm">⚠️ {error}</Text>
              </Box>
            )}
          </VStack>

          {/* Right Panel - Always Present */}
          <VStack gap={4} bg="gray.800" p={4} borderRadius="lg" flex="1" minH="400px" maxW="50%">
            {deviceVerified && addressState ? (
              /* Address Display When Verified */
              <>
                {/* Address Info Header */}
                <HStack w="100%" justify="space-between" align="center">
                  <VStack align="start" gap={1}>
                    <Text color="green.300" fontSize="sm" fontWeight="bold">
                      ✅ Device Verified Address
                    </Text>
                    <Text color="gray.400" fontSize="xs" fontFamily="mono">
                      {addressState.derivationPath}
                    </Text>
                  </VStack>
                  <VStack align="end" gap={1}>
                                      <Text color="gray.300" fontSize="xs" bg="gray.700" px={2} py={1} borderRadius="md">
                    Index: {addressState.addressIndex}
                  </Text>
                  <Text color="green.300" fontSize="xs" bg="gray.700" px={2} py={1} borderRadius="md">
                    Receive
                  </Text>
                  </VStack>
                </HStack>

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
                    value={addressState.address}
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
                  <HStack w="100%">
                    <Input
                      value={addressState.address}
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
                    colorScheme={hasCopied ? "green" : "gray"}
                    size="sm"
                  >
                      {hasCopied ? <FaCheck /> : <FaCopy />}
                    </IconButton>
                  </HStack>
                  {hasCopied && (
                    <Text fontSize="xs" color="green.400">
                      Address copied to clipboard!
                    </Text>
                  )}
                </VStack>
              </>
            ) : (
              /* Device Verification In Place of QR Code */
              <>
                {/* Path Info Header Even When Not Verified */}
                <HStack w="100%" justify="space-between" align="center">
                  <VStack align="start" gap={1}>
                    <Text color="gray.400" fontSize="sm" fontWeight="bold">
                      Address Preview
                    </Text>
                    <Text color="gray.400" fontSize="xs" fontFamily="mono">
                      {formatDerivationPath(
                        walletState.selectedScriptType,
                        walletState.selectedAccount, 
                        walletState.showChangeAddresses, 
                        walletState.currentAddressIndex
                      )}
                    </Text>
                  </VStack>
                  <VStack align="end" gap={1}>
                    <Text color="gray.300" fontSize="xs" bg="gray.700" px={2} py={1} borderRadius="md">
                      Index: {walletState.currentAddressIndex}
                    </Text>
                    <Text color="gray.400" fontSize="xs" bg="gray.700" px={2} py={1} borderRadius="md">
                      Pending
                    </Text>
                  </VStack>
                </HStack>

                {/* Device Verification Instead of QR Code */}
                <VStack 
                  gap={4} 
                  bg="gray.700" 
                  p={4} 
                  borderRadius="lg" 
                  border="2px solid" 
                  borderColor="gray.500" 
                  w="100%"
                  minH="200px"
                  justify="center"
                >
                  <HStack gap={2} align="center">
                    <Box color="gray.300" fontSize="2xl">
                      <FiShield />
                    </Box>
                    <Text color="white" fontWeight="bold" fontSize="lg">
                      Device Verification
                    </Text>
                  </HStack>
                  <Text color="gray.300" fontSize="sm" textAlign="center">
                    Verify address on your KeepKey device to display QR code
                  </Text>
                  <Button
                    colorScheme="gray"
                    size="lg"
                    onClick={verifyOnDevice}
                    disabled={verifying}
                    w="100%"
                    bg="gray.600"
                    _hover={{ bg: "gray.500" }}
                    color="white"
                  >
                    <HStack gap={2}>
                      {verifying ? <Spinner size="sm" /> : <FaEye />}
                      <Text>{verifying ? 'Verifying...' : 'Verify on KeepKey'}</Text>
                    </HStack>
                  </Button>
                </VStack>

                {/* Address Placeholder */}
                <VStack w="100%" gap={2}>
                  <Text color="gray.300" fontSize="sm" fontWeight="medium">
                    Bitcoin Address
                  </Text>
                  <Box
                    w="100%"
                    p={3}
                    bg="gray.700"
                    borderRadius="md"
                    border="1px solid"
                    borderColor="gray.600"
                    textAlign="center"
                  >
                    <Text color="gray.500" fontSize="sm">
                      Address will appear after device verification
                    </Text>
                  </Box>
                </VStack>
              </>
            )}
          </VStack>
        </HStack>

        {/* Bottom Navigation */}
        <HStack justify="center">
          <Button
            variant="ghost"
            color="gray.400"
            onClick={onBack}
            size="lg"
          >
            Done
          </Button>
        </HStack>


              </VStack>
      </Box>
    </Box>
  );
};

export default Receive;
