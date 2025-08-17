import React from 'react';
import { useTranslation } from 'react-i18next';
import { Box } from '@chakra-ui/react';
import { SUPPORTED_LANGUAGES } from '../i18n/languages';

export const LanguageSwitcher: React.FC = () => {
  const { i18n } = useTranslation();

  const handleLanguageChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    const languageCode = event.target.value;
    i18n.changeLanguage(languageCode);
    localStorage.setItem('preferredLanguage', languageCode);
  };

  return (
    <Box>
      <select
        value={i18n.language}
        onChange={handleLanguageChange}
        style={{
          backgroundColor: '#2D3748',
          color: 'white',
          padding: '4px 8px',
          borderRadius: '4px',
          border: '1px solid #4A5568',
          fontSize: '14px',
          cursor: 'pointer'
        }}
      >
        {SUPPORTED_LANGUAGES.map((lang) => (
          <option key={lang.code} value={lang.code} style={{ color: 'black', backgroundColor: 'white' }}>
            {lang.flag} {lang.nativeName}
          </option>
        ))}
      </select>
    </Box>
  );
};