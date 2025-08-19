import { Box, Card, HStack, Text, VStack, Icon } from "@chakra-ui/react";
import { FaShieldAlt, FaLock, FaKey } from "react-icons/fa";
import { useTranslation } from "react-i18next";

// Step components no longer need props - navigation handled by main wizard

export function Step2Pin() {
  const { t } = useTranslation(['onboarding']);

  return (
    <Box width="full" maxWidth="lg">
      <Card.Root bg="gray.900" borderColor="gray.700">
        <Card.Header bg="gray.850">
          <HStack justify="center" gap={3}>
            <Icon asChild color="green.500">
              <FaShieldAlt />
            </Icon>
            <Text fontSize="xl" fontWeight="bold" color="white">
              {t('onboarding:pin.subtitle')}
            </Text>
          </HStack>
        </Card.Header>
        <Card.Body>
          <VStack gap={3}>
            <Text color="gray.400" textAlign="center">
              {t('onboarding:pin.fullDescription')}
            </Text>
            
            <HStack gap={6} width="full" alignItems="stretch">
              <Box flex="1" p={3} bg="gray.800" borderRadius="md" borderWidth="2px" borderColor="green.600">
                <HStack gap={3} mb={3}>
                  <Icon asChild color="green.400">
                    <FaLock />
                  </Icon>
                  <Text color="white" fontWeight="medium">{t('onboarding:pin.pinProtection.title')}</Text>
                </HStack>
                <Text color="gray.400" fontSize="sm" mb={2}>
                  {t('onboarding:pin.pinProtection.description')}
                </Text>
                <Text color="green.400" fontSize="sm" fontWeight="medium">
                  {t('onboarding:pin.pinProtection.recommendation')}
                </Text>
              </Box>
              
              <Box flex="1" p={3} bg="gray.800" borderRadius="md" borderWidth="2px" borderColor="orange.600">
                <HStack gap={3} mb={3}>
                  <Icon asChild color="orange.400">
                    <FaKey />
                  </Icon>
                  <Text color="white" fontWeight="medium">{t('onboarding:pin.recoveryPhrase.title')}</Text>
                </HStack>
                <Text color="gray.400" fontSize="sm" mb={2}>
                  {t('onboarding:pin.recoveryPhrase.description')}
                </Text>
                <Text color="orange.400" fontSize="sm" fontWeight="medium">
                  {t('onboarding:pin.recoveryPhrase.instruction')}
                </Text>
              </Box>
            </HStack>
            
            <Box textAlign="center" p={3} bg="blue.900" borderRadius="md" borderWidth="1px" borderColor="blue.600">
              <Text color="blue.400" fontSize="sm">
                {t('onboarding:pin.setupInfo')}
              </Text>
            </Box>
          </VStack>
        </Card.Body>
      </Card.Root>
    </Box>
  );
} 