import { VStack, Text, Button, Box, Icon, HStack } from "@chakra-ui/react";
import { FaPlus, FaKey } from "react-icons/fa";
import { useTypedTranslation } from "../../../hooks/useTypedTranslation";

interface Step1CreateOrRecoverProps {
  onFlowTypeSelect: (type: 'create' | 'recover') => void;
}

export function Step1CreateOrRecover({ onFlowTypeSelect }: Step1CreateOrRecoverProps) {
  const { t } = useTypedTranslation('setup');
  
  return (
    <VStack gap={{ base: 4, md: 6, lg: 8 }} w="100%">
      <VStack gap={2}>
        <Text fontSize={{ base: "xl", md: "2xl" }} fontWeight="bold" color="white" textAlign="center">
          {t('setupMethod.title')}
        </Text>
        <Text fontSize={{ base: "sm", md: "md" }} color="gray.400" textAlign="center">
          {t('setupMethod.subtitle')}
        </Text>
      </VStack>

      <HStack 
        gap={{ base: 4, md: 6 }} 
        w="100%" 
        flexDirection={{ base: "column", md: "row" }}
        align="stretch"
      >
        {/* Create New Wallet */}
        <Box
          flex={{ base: "none", md: 1 }}
          w={{ base: "100%", md: "auto" }}
          maxW={{ base: "400px", md: "none" }}
          p={{ base: 4, md: 6 }}
          borderRadius="lg"
          borderWidth="2px"
          borderColor="transparent"
          bg="gray.700"
          cursor="pointer"
          transition="all 0.2s"
          _hover={{
            borderColor: "orange.500",
            bg: "gray.650",
            transform: "translateY(-2px)",
          }}
          onClick={() => onFlowTypeSelect('create')}
        >
          <VStack gap={4}>
            <Box
              p={{ base: 3, md: 4 }}
              borderRadius="full"
              bg="orange.500"
              color="white"
            >
              <Icon as={FaPlus} boxSize={{ base: 6, md: 8 }} />
            </Box>
            <VStack gap={2}>
              <Text fontSize={{ base: "lg", md: "xl" }} fontWeight="bold" color="white">
                {t('setupMethod.createWallet.title')}
              </Text>
              <Text fontSize={{ base: "xs", md: "sm" }} color="gray.400" textAlign="center">
                {t('setupMethod.createWallet.description')}
              </Text>
            </VStack>
            <Button
              colorScheme="orange"
              size={{ base: "md", md: "lg" }}
              w="100%"
              onClick={(e) => {
                e.stopPropagation();
                onFlowTypeSelect('create');
              }}
            >
              {t('setupMethod.createWallet.button')}
            </Button>
          </VStack>
        </Box>

        {/* Recover Existing Wallet */}
        <Box
          flex={{ base: "none", md: 1 }}
          w={{ base: "100%", md: "auto" }}
          maxW={{ base: "400px", md: "none" }}
          p={{ base: 4, md: 6 }}
          borderRadius="lg"
          borderWidth="2px"
          borderColor="transparent"
          bg="gray.700"
          cursor="pointer"
          transition="all 0.2s"
          _hover={{
            borderColor: "blue.500",
            bg: "gray.650",
            transform: "translateY(-2px)",
          }}
          onClick={() => onFlowTypeSelect('recover')}
        >
          <VStack gap={4}>
            <Box
              p={{ base: 3, md: 4 }}
              borderRadius="full"
              bg="blue.500"
              color="white"
            >
              <Icon as={FaKey} boxSize={{ base: 6, md: 8 }} />
            </Box>
            <VStack gap={2}>
              <Text fontSize={{ base: "lg", md: "xl" }} fontWeight="bold" color="white">
                {t('setupMethod.recoverWallet.title')}
              </Text>
              <Text fontSize={{ base: "xs", md: "sm" }} color="gray.400" textAlign="center">
                {t('setupMethod.recoverWallet.description')}
              </Text>
            </VStack>
            <Button
              colorScheme="blue"
              size={{ base: "md", md: "lg" }}
              w="100%"
              onClick={(e) => {
                e.stopPropagation();
                onFlowTypeSelect('recover');
              }}
            >
              {t('setupMethod.recoverWallet.button')}
            </Button>
          </VStack>
        </Box>
      </HStack>
    </VStack>
  );
}