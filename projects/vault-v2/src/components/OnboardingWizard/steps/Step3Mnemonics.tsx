import { Box, Card, HStack, Text, VStack, Icon } from "@chakra-ui/react";
import { FaBook, FaExclamationTriangle, FaCheckCircle } from "react-icons/fa";
import { useTranslation } from "react-i18next";

// Step components no longer need props - navigation handled by main wizard

export function Step3Mnemonics() {
  const { t } = useTranslation(['onboarding']);

  return (
    <Box width="full" maxWidth="2xl">
      <Card.Root bg="gray.900" borderColor="gray.700">
        <Card.Header bg="gray.850">
          <HStack justify="center" gap={3}>
            <Icon asChild color="green.500">
              <FaBook />
            </Icon>
            <Text fontSize="xl" fontWeight="bold" color="white">
              {t('onboarding:mnemonics.subtitle')}
            </Text>
          </HStack>
        </Card.Header>
        <Card.Body>
          <VStack gap={4}>
            <Text color="gray.400" textAlign="center">
              {t('onboarding:mnemonics.fullDescription')}
            </Text>
            
            <HStack gap={4} alignItems="stretch" width="full">
              <Box flex="1" p={3} bg="red.900" borderRadius="md" borderColor="red.700" borderWidth="2px">
                <HStack gap={3} mb={3}>
                  <Icon asChild color="red.400">
                    <FaExclamationTriangle />
                  </Icon>
                  <Text color="white" fontWeight="medium">{t('onboarding:mnemonics.securityWarning.title')}</Text>
                </HStack>
                <Text color="red.200" fontSize="sm" mb={2}>
                  {t('onboarding:mnemonics.securityWarning.description')}
                </Text>
                <Text color="red.300" fontSize="sm" fontWeight="bold">
                  {t('onboarding:mnemonics.securityWarning.strongWarning')}
                </Text>
              </Box>
              
              <Box flex="1" p={3} bg="gray.800" borderRadius="md" borderWidth="2px" borderColor="green.600">
                <HStack gap={3} mb={3}>
                  <Icon asChild color="green.400">
                    <FaCheckCircle />
                  </Icon>
                  <Text color="white" fontWeight="medium">{t('onboarding:mnemonics.whatIs.title')}</Text>
                </HStack>
                <VStack align="start" gap={2}>
                  <Text color="gray.400" fontSize="sm">
                    {t('onboarding:mnemonics.whatIs.point1')}
                  </Text>
                  <Text color="gray.400" fontSize="sm">
                    {t('onboarding:mnemonics.whatIs.point2')}
                  </Text>
                  <Text color="gray.400" fontSize="sm">
                    {t('onboarding:mnemonics.whatIs.point3')}
                  </Text>
                  <Text color="gray.400" fontSize="sm">
                    {t('onboarding:mnemonics.whatIs.point4')}
                  </Text>
                </VStack>
              </Box>
            </HStack>
            
            <Box textAlign="center" p={3} bg="yellow.900" borderRadius="md" borderWidth="1px" borderColor="yellow.600">
              <Text color="yellow.400" fontSize="sm" fontWeight="medium">
                {t('onboarding:mnemonics.preparation')}
              </Text>
            </Box>
          </VStack>
        </Card.Body>
      </Card.Root>
    </Box>
  );
} 