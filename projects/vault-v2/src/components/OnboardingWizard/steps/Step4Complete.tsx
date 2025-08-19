import { Box, Card, HStack, Text, VStack, Icon } from "@chakra-ui/react";
import { FaCheckCircle, FaRocket } from "react-icons/fa";
import { useTranslation } from "react-i18next";

// Step components no longer need props - navigation handled by main wizard

export function Step4Complete() {
  const { t } = useTranslation(['onboarding']);

  return (
    <Box width="full" maxWidth="lg">
      <Card.Root bg="gray.900" borderColor="gray.700">
        <Card.Body>
          <VStack gap={3} align="center" textAlign="center">
            <Icon asChild color="green.500" boxSize={12}>
              <FaCheckCircle />
            </Icon>
            
            <VStack gap={2}>
              <Text fontSize="2xl" fontWeight="bold" color="green.400">
                {t('onboarding:complete.title')}
              </Text>
              <Text color="gray.400" fontSize="lg">
                {t('onboarding:complete.subtitle')}
              </Text>
              <Text color="gray.300">
                {t('onboarding:complete.message')}
              </Text>
            </VStack>
            
            <Box w="full" mt={3} p={4} bg="gradient-to-r from-blue.900 to-green.900" borderRadius="lg" borderWidth="2px" borderColor="green.600">
              <HStack gap={3} justify="center" mb={3}>
                <Icon asChild color="yellow.400" boxSize={6}>
                  <FaRocket />
                </Icon>
                <Text color="white" fontWeight="bold" fontSize="lg">{t('onboarding:complete.nextSteps.title')}</Text>
              </HStack>
              <VStack gap={2}>
                <Text color="green.200" fontSize="sm" fontWeight="medium">
                  {t('onboarding:complete.nextSteps.step1')}
                </Text>
                <Text color="green.200" fontSize="sm" fontWeight="medium">
                  {t('onboarding:complete.nextSteps.step2')}
                </Text>
                <Text color="green.200" fontSize="sm" fontWeight="medium">
                  {t('onboarding:complete.nextSteps.step3')}
                </Text>
              </VStack>
            </Box>
            
            <Box textAlign="center" p={3} bg="gray.800" borderRadius="md" borderWidth="1px" borderColor="gray.600">
              <Text color="gray.400" fontSize="sm">
                {t('onboarding:complete.help')}
              </Text>
            </Box>
          </VStack>
        </Card.Body>
      </Card.Root>
    </Box>
  );
} 