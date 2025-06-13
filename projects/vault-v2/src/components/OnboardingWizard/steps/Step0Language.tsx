import {
  Box,
  Button,
  Card,
  HStack,
  Text,
  VStack,
  Icon,
} from "@chakra-ui/react";
import { FaGlobe } from "react-icons/fa";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface StepProps {
  onNext: () => void;
  onPrevious: () => void;
}

const LANGUAGES = [
  { key: "en", label: "English" },
  { key: "es", label: "Español" },
  { key: "fr", label: "Français" },
  { key: "de", label: "Deutsch" },
  { key: "it", label: "Italiano" },
  { key: "pt", label: "Português" },
  { key: "ru", label: "Русский" },
  { key: "zh", label: "中文" },
  { key: "ja", label: "日本語" },
  { key: "ko", label: "한국어" },
];

export function Step0Language({ onNext }: StepProps) {
  const [selectedLanguage, setSelectedLanguage] = useState("en");

  useEffect(() => {
    // Load saved language preference
    loadLanguagePreference();
  }, []);

  const loadLanguagePreference = async () => {
    try {
      const savedLang = await invoke<string | null>("get_preference", { key: "language" });
      if (savedLang) {
        setSelectedLanguage(savedLang);
      }
    } catch (error) {
      console.error("Failed to load language preference:", error);
    }
  };

  const handleLanguageChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setSelectedLanguage(event.target.value);
  };

  const handleNext = async () => {
    try {
      // Save language preference
      await invoke("set_preference", { key: "language", value: selectedLanguage });
      onNext();
    } catch (error) {
      console.error("Failed to save language preference:", error);
    }
  };

  return (
    <VStack align="center" justify="center" minH="400px" gap={6}>
      <Card.Root width="full" maxWidth="md" bg="gray.900" borderColor="gray.700">
        <Card.Header bg="gray.850">
          <HStack justify="center" gap={3}>
            <Icon asChild color="green.500">
              <FaGlobe />
            </Icon>
            <Text fontSize="xl" fontWeight="bold" color="white">
              Select Your Language
            </Text>
          </HStack>
        </Card.Header>
        <Card.Body>
          <VStack gap={4}>
            <Text color="gray.400">
              Choose your preferred language for the KeepKey Desktop application.
            </Text>
            <select
              value={selectedLanguage}
              onChange={handleLanguageChange}
              style={{
                width: '100%',
                padding: '0.5rem',
                fontSize: '1rem',
                borderRadius: '0.375rem',
                border: '1px solid #4A5568',
                backgroundColor: '#2D3748',
                color: '#E2E8F0'
              }}
            >
              {LANGUAGES.map((lang) => (
                <option key={lang.key} value={lang.key} style={{ backgroundColor: '#2D3748' }}>
                  {lang.label}
                </option>
              ))}
            </select>
          </VStack>
        </Card.Body>
      </Card.Root>

      <HStack gap={4}>
        <Button 
          variant="outline" 
          disabled
          borderColor="gray.600"
          color="gray.500"
          _hover={{ bg: "gray.700" }}
        >
          Previous
        </Button>
        <Button colorScheme="green" onClick={handleNext}>
          Next
        </Button>
      </HStack>
    </VStack>
  );
} 