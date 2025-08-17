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

// Spanish translations
import esCommon from './locales/es/common.json';
import esOnboarding from './locales/es/onboarding.json';
import esSettings from './locales/es/settings.json';
import esErrors from './locales/es/errors.json';
import esDevice from './locales/es/device.json';
import esWallet from './locales/es/wallet.json';
import esTransaction from './locales/es/transaction.json';

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
  es: {
    common: esCommon,
    onboarding: esOnboarding,
    settings: esSettings,
    errors: esErrors,
    device: esDevice,
    wallet: esWallet,
    transaction: esTransaction,
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
    
    debug: true, // Enable debug mode to see what's happening
  }, (err, t) => {
    if (err) {
      console.error('🌐 i18n initialization failed:', err);
    } else {
      console.log('🌐 i18n initialized successfully');
      console.log('🌐 Available languages:', Object.keys(resources));
      console.log('🌐 Current language:', i18n.language);
    }
  });

export default i18n;