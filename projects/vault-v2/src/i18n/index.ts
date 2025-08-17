import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

// Import translation files
import enCommon from './locales/en/common.json';
import enOnboarding from './locales/en/onboarding.json';
import enSettings from './locales/en/settings.json';
import enErrors from './locales/en/errors.json';
import enDevice from './locales/en/device.json';
import enWallet from './locales/en/wallet.json';
import enTransaction from './locales/en/transaction.json';

const resources = {
  en: {
    common: enCommon,
    onboarding: enOnboarding,
    settings: enSettings,
    errors: enErrors,
    device: enDevice,
    wallet: enWallet,
    transaction: enTransaction,
  },
};

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: 'en',
    defaultNS: 'common',
    ns: ['common', 'onboarding', 'settings', 'errors', 'device', 'wallet', 'transaction'],
    
    interpolation: {
      escapeValue: false, // React already escapes values
    },
    
    detection: {
      order: ['localStorage', 'navigator'],
      caches: ['localStorage'],
    },
    
    react: {
      useSuspense: false, // Disable suspense for better error handling
    },
  });

export default i18n;