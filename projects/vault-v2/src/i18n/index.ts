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
import enDialogs from './locales/en/dialogs.json';
import enSetup from './locales/en/setup.json';

// Spanish translations
import esCommon from './locales/es/common.json';
import esOnboarding from './locales/es/onboarding.json';
import esSettings from './locales/es/settings.json';
import esErrors from './locales/es/errors.json';
import esDevice from './locales/es/device.json';
import esWallet from './locales/es/wallet.json';
import esTransaction from './locales/es/transaction.json';
import esDialogs from './locales/es/dialogs.json';
import esSetup from './locales/es/setup.json';

// French translations
import frCommon from './locales/fr/common.json';
import frOnboarding from './locales/fr/onboarding.json';
import frSettings from './locales/fr/settings.json';
import frErrors from './locales/fr/errors.json';
import frDevice from './locales/fr/device.json';
import frWallet from './locales/fr/wallet.json';
import frTransaction from './locales/fr/transaction.json';
import frDialogs from './locales/fr/dialogs.json';
import frSetup from './locales/fr/setup.json';

// German translations
import deCommon from './locales/de/common.json';
import deOnboarding from './locales/de/onboarding.json';
import deSettings from './locales/de/settings.json';
import deErrors from './locales/de/errors.json';
import deDevice from './locales/de/device.json';
import deWallet from './locales/de/wallet.json';
import deTransaction from './locales/de/transaction.json';
import deDialogs from './locales/de/dialogs.json';
import deSetup from './locales/de/setup.json';

// Polish translations
import plCommon from './locales/pl/common.json';
import plOnboarding from './locales/pl/onboarding.json';
import plSettings from './locales/pl/settings.json';
import plErrors from './locales/pl/errors.json';
import plDevice from './locales/pl/device.json';
import plWallet from './locales/pl/wallet.json';
import plTransaction from './locales/pl/transaction.json';
import plDialogs from './locales/pl/dialogs.json';
import plSetup from './locales/pl/setup.json';

const resources = {
  en: {
    common: enCommon,
    onboarding: enOnboarding,
    settings: enSettings,
    errors: enErrors,
    device: enDevice,
    wallet: enWallet,
    transaction: enTransaction,
    dialogs: enDialogs,
    setup: enSetup,
  },
  es: {
    common: esCommon,
    onboarding: esOnboarding,
    settings: esSettings,
    errors: esErrors,
    device: esDevice,
    wallet: esWallet,
    transaction: esTransaction,
    dialogs: esDialogs,
    setup: esSetup,
  },
  fr: {
    common: frCommon,
    onboarding: frOnboarding,
    settings: frSettings,
    errors: frErrors,
    device: frDevice,
    wallet: frWallet,
    transaction: frTransaction,
    dialogs: frDialogs,
    setup: frSetup,
  },
  de: {
    common: deCommon,
    onboarding: deOnboarding,
    settings: deSettings,
    errors: deErrors,
    device: deDevice,
    wallet: deWallet,
    transaction: deTransaction,
    dialogs: deDialogs,
    setup: deSetup,
  },
  pl: {
    common: plCommon,
    onboarding: plOnboarding,
    settings: plSettings,
    errors: plErrors,
    device: plDevice,
    wallet: plWallet,
    transaction: plTransaction,
    dialogs: plDialogs,
    setup: plSetup,
  },
};

// Get saved language preference
const savedLanguage = localStorage.getItem('preferredLanguage') || 'en';
console.log('🌐 Loading saved language preference:', savedLanguage);

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    lng: savedLanguage, // Set initial language from localStorage
    fallbackLng: 'en',
    defaultNS: 'common',
    ns: ['common', 'onboarding', 'settings', 'errors', 'device', 'wallet', 'transaction', 'dialogs', 'setup'],
    
    interpolation: {
      escapeValue: false, // React already escapes values
    },
    
    detection: {
      order: ['localStorage', 'navigator'],
      caches: ['localStorage'],
      lookupLocalStorage: 'preferredLanguage', // Use our custom key
    },
    
    react: {
      useSuspense: false, // Disable suspense for better error handling
    },
    
    debug: false, // Disable debug mode for cleaner console
  }, (err, t) => {
    if (err) {
      console.error('🌐 i18n initialization failed:', err);
    } else {
      console.log('🌐 i18n initialized successfully');
      console.log('🌐 Available languages:', Object.keys(resources));
      console.log('🌐 Current language:', i18n.language);
      
      // Ensure the saved language is applied
      if (savedLanguage && savedLanguage !== i18n.language) {
        i18n.changeLanguage(savedLanguage);
      }
    }
  });

export default i18n;