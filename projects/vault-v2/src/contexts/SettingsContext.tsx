import React, { createContext, useContext, useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { getStoredCurrency, getStoredNumberFormat } from '../utils/currency';

export type BitcoinAddressType = 'p2wpkh' | 'p2sh-p2wpkh' | 'p2pkh';

export interface SettingsContextType {
  currency: string;
  numberFormat: string;
  language: string;
  bitcoinAddressType: BitcoinAddressType;
  setCurrency: (currency: string) => void;
  setNumberFormat: (format: string) => void;
  setLanguage: (language: string) => void;
  setBitcoinAddressType: (type: BitcoinAddressType) => void;
}

const SettingsContext = createContext<SettingsContextType | undefined>(undefined);

export const useSettings = () => {
  const context = useContext(SettingsContext);
  if (!context) {
    throw new Error('useSettings must be used within a SettingsProvider');
  }
  return context;
};

interface SettingsProviderProps {
  children: React.ReactNode;
}

export const SettingsProvider: React.FC<SettingsProviderProps> = ({ children }) => {
  const { i18n } = useTranslation();
  const [currency, setCurrencyState] = useState<string>(getStoredCurrency());
  const [numberFormat, setNumberFormatState] = useState<string>(getStoredNumberFormat());
  const [language, setLanguageState] = useState<string>(i18n.language);
  const [bitcoinAddressType, setBitcoinAddressTypeState] = useState<BitcoinAddressType>(() => {
    const stored = localStorage.getItem('bitcoinAddressType');
    return (stored as BitcoinAddressType) || 'p2wpkh'; // Default to Native SegWit
  });

  // Sync with i18n language changes
  useEffect(() => {
    setLanguageState(i18n.language);
  }, [i18n.language]);

  const setCurrency = (newCurrency: string) => {
    setCurrencyState(newCurrency);
    localStorage.setItem('preferredCurrency', newCurrency);
  };

  const setNumberFormat = (newFormat: string) => {
    setNumberFormatState(newFormat);
    localStorage.setItem('numberFormat', newFormat);
  };

  const setLanguage = (newLanguage: string) => {
    setLanguageState(newLanguage);
    i18n.changeLanguage(newLanguage);
    localStorage.setItem('preferredLanguage', newLanguage);
  };

  const setBitcoinAddressType = (newType: BitcoinAddressType) => {
    setBitcoinAddressTypeState(newType);
    localStorage.setItem('bitcoinAddressType', newType);
  };

  const value: SettingsContextType = {
    currency,
    numberFormat,
    language,
    bitcoinAddressType,
    setCurrency,
    setNumberFormat,
    setLanguage,
    setBitcoinAddressType,
  };

  return (
    <SettingsContext.Provider value={value}>
      {children}
    </SettingsContext.Provider>
  );
};