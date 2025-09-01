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
import { useTranslation } from "react-i18next";
import i18n from "../../../i18n";

// Step components no longer need props - navigation handled by main wizard

// Only show languages that are actually supported with full translations
const LANGUAGES = [
  { key: "en", label: "English" },
  { key: "es", label: "Español" },
  { key: "fr", label: "Français" },
  { key: "de", label: "Deutsch" },
];

export function Step0Language() {
  // Initialize with current i18n language instead of hardcoded "en"
  const [selectedLanguage, setSelectedLanguage] = useState(i18n.language || "en");
  const { t } = useTranslation(['onboarding']);

  useEffect(() => {
    // Set initial language to current i18n language
    setSelectedLanguage(i18n.language || "en");
    
    // Load saved language preference (this might override the above)
    loadLanguagePreference();
  }, []);

  const loadLanguagePreference = async () => {
    try {
      const savedLang = await invoke<string | null>("get_preference", { key: "language" });
      if (savedLang && LANGUAGES.some(lang => lang.key === savedLang)) {
        setSelectedLanguage(savedLang);
        // Apply the saved language to i18n if different from current
        if (i18n.language !== savedLang) {
          await i18n.changeLanguage(savedLang);
          console.log("Applied saved language preference:", savedLang);
        }
      }
    } catch (error) {
      console.error("Failed to load language preference:", error);
    }
  };

  const handleLanguageChange = async (event: React.ChangeEvent<HTMLSelectElement>) => {
    const newLanguage = event.target.value;
    setSelectedLanguage(newLanguage);
    
    // Immediately change the language in i18n to update translations
    await i18n.changeLanguage(newLanguage);
    console.log("Language changed to:", newLanguage);
    
    // Auto-save language preference when changed
    handleLanguageSave(newLanguage);
  };

  const handleLanguageSave = async (language: string) => {
    try {
      // Save language preference to backend
      await invoke("set_preference", { key: "language", value: language });
      console.log("Language preference saved to backend:", language);
      
      // Also save to localStorage for i18n persistence
      localStorage.setItem('preferredLanguage', language);
      console.log("Language preference saved to localStorage:", language);
    } catch (error) {
      console.error("Failed to save language preference:", error);
    }
  };

  // Language preference is automatically saved when changed
  // Navigation is handled by the main wizard component

  return (
    <Box width="full" maxWidth="lg">
      <Card.Root bg="gray.900" borderColor="gray.700">
        <Card.Header bg="gray.850">
          <HStack justify="center" gap={3}>
            <Icon asChild color="green.500">
              <FaGlobe />
            </Icon>
            <Text fontSize="xl" fontWeight="bold" color="white">
              {t('onboarding:language.title')}
            </Text>
          </HStack>
        </Card.Header>
        <Card.Body>
          <VStack gap={6}>
            <Text color="gray.400" textAlign="center">
              {t('onboarding:language.fullDescription')}
            </Text>
            
            <Box width="full" maxWidth="md">
              <Text color="white" fontSize="sm" mb={2} fontWeight="medium">
                Language / Idioma / Langue / Sprache
              </Text>
              <select
                value={selectedLanguage}
                onChange={handleLanguageChange}
                style={{
                  width: '100%',
                  padding: '0.75rem',
                  fontSize: '1rem',
                  borderRadius: '0.5rem',
                  border: '2px solid #4A5568',
                  backgroundColor: '#2D3748',
                  color: '#E2E8F0',
                  cursor: 'pointer'
                }}
              >
                {LANGUAGES.map((lang) => (
                  <option key={lang.key} value={lang.key} style={{ backgroundColor: '#2D3748' }}>
                    {lang.label}
                  </option>
                ))}
              </select>
            </Box>
            
            <Box textAlign="center" mt={4}>
              <Text color="green.400" fontSize="sm">
                ✓ {t('onboarding:language.appliedImmediately')}
              </Text>
            </Box>
          </VStack>
        </Card.Body>
      </Card.Root>
    </Box>
  );
} 