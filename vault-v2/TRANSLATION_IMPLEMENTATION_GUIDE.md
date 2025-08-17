# KeepKey Vault-v2 Translation Implementation Guide

## Quick Start

This guide provides step-by-step instructions for implementing internationalization (i18n) in the vault-v2 application.

## Step 1: Install Dependencies

```bash
cd vault-v2
npm install i18next react-i18next i18next-browser-languagedetector i18next-http-backend
npm install --save-dev @types/react-i18next
```

## Step 2: Create i18n Configuration

### Create `src/i18n/index.ts`
```typescript
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

// Import translation files
import enCommon from './locales/en/common.json';
import enOnboarding from './locales/en/onboarding.json';
import enSettings from './locales/en/settings.json';
import enErrors from './locales/en/errors.json';
import enDevice from './locales/en/device.json';

const resources = {
  en: {
    common: enCommon,
    onboarding: enOnboarding,
    settings: enSettings,
    errors: enErrors,
    device: enDevice,
  },
};

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: 'en',
    defaultNS: 'common',
    ns: ['common', 'onboarding', 'settings', 'errors', 'device'],
    
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
```

### Create `src/i18n/types.ts`
```typescript
import 'react-i18next';
import type common from './locales/en/common.json';
import type onboarding from './locales/en/onboarding.json';
import type settings from './locales/en/settings.json';
import type errors from './locales/en/errors.json';
import type device from './locales/en/device.json';

declare module 'react-i18next' {
  interface CustomTypeOptions {
    defaultNS: 'common';
    resources: {
      common: typeof common;
      onboarding: typeof onboarding;
      settings: typeof settings;
      errors: typeof errors;
      device: typeof device;
    };
  }
}
```

### Create `src/i18n/languages.ts`
```typescript
export interface Language {
  code: string;
  name: string;
  nativeName: string;
  flag: string;
  rtl?: boolean;
}

export const SUPPORTED_LANGUAGES: Language[] = [
  { code: 'en', name: 'English', nativeName: 'English', flag: '🇺🇸' },
  { code: 'es', name: 'Spanish', nativeName: 'Español', flag: '🇪🇸' },
  { code: 'fr', name: 'French', nativeName: 'Français', flag: '🇫🇷' },
  { code: 'de', name: 'German', nativeName: 'Deutsch', flag: '🇩🇪' },
  { code: 'ja', name: 'Japanese', nativeName: '日本語', flag: '🇯🇵' },
  { code: 'zh', name: 'Chinese', nativeName: '中文', flag: '🇨🇳' },
  { code: 'ko', name: 'Korean', nativeName: '한국어', flag: '🇰🇷' },
  { code: 'pt', name: 'Portuguese', nativeName: 'Português', flag: '🇧🇷' },
];

export const DEFAULT_LANGUAGE = 'en';
```

## Step 3: Create Translation Files

### Create `src/i18n/locales/en/common.json`
```json
{
  "buttons": {
    "cancel": "Cancel",
    "confirm": "Confirm",
    "save": "Save",
    "close": "Close",
    "back": "Back",
    "next": "Next",
    "finish": "Finish",
    "retry": "Retry",
    "continue": "Continue",
    "copy": "Copy",
    "copied": "Copied!",
    "enable": "Enable",
    "disable": "Disable",
    "refresh": "Refresh",
    "download": "Download",
    "upload": "Upload",
    "delete": "Delete",
    "edit": "Edit",
    "update": "Update"
  },
  "status": {
    "loading": "Loading...",
    "processing": "Processing...",
    "connecting": "Connecting...",
    "syncing": "Syncing...",
    "error": "Error",
    "success": "Success",
    "warning": "Warning",
    "info": "Information",
    "ready": "Ready",
    "disconnected": "Disconnected",
    "connected": "Connected"
  },
  "navigation": {
    "home": "Home",
    "settings": "Settings",
    "help": "Help",
    "about": "About",
    "portfolio": "Portfolio",
    "transactions": "Transactions",
    "receive": "Receive",
    "send": "Send"
  },
  "time": {
    "seconds": "seconds",
    "minutes": "minutes",
    "hours": "hours",
    "days": "days",
    "ago": "ago"
  }
}
```

### Create `src/i18n/locales/en/onboarding.json`
```json
{
  "language": {
    "title": "Select Your Language",
    "description": "Choose your preferred language for the KeepKey Desktop application",
    "changeAnytime": "You can change this anytime in settings"
  },
  "welcome": {
    "title": "Welcome to KeepKey",
    "subtitle": "Your journey to secure crypto management starts here",
    "getStarted": "Get Started"
  },
  "setup": {
    "title": "Setup Your Device",
    "connectDevice": "Please connect your KeepKey device",
    "detectingDevice": "Detecting device...",
    "deviceFound": "Device found!",
    "troubleshooting": "Having trouble? Check our troubleshooting guide"
  }
}
```

### Create `src/i18n/locales/en/settings.json`
```json
{
  "title": "Settings",
  "tabs": {
    "general": "General",
    "security": "Security",
    "advanced": "Advanced",
    "about": "About"
  },
  "general": {
    "language": {
      "label": "Display Language",
      "description": "Choose your preferred language"
    },
    "theme": {
      "label": "Theme",
      "description": "Choose between light and dark mode",
      "light": "Light",
      "dark": "Dark",
      "system": "System"
    },
    "currency": {
      "label": "Primary Currency",
      "description": "Display values in your preferred currency"
    }
  },
  "security": {
    "pin": {
      "label": "PIN Settings",
      "change": "Change PIN",
      "remove": "Remove PIN"
    },
    "passphrase": {
      "label": "Passphrase",
      "enable": "Enable Passphrase",
      "disable": "Disable Passphrase"
    }
  }
}
```

### Create `src/i18n/locales/en/errors.json`
```json
{
  "device": {
    "notConnected": "No KeepKey device connected",
    "connectionFailed": "Failed to connect to device",
    "connectionLost": "Connection to device lost",
    "busy": "Device is busy, please wait",
    "locked": "Device is locked. Please enter your PIN",
    "bootloaderMode": "Device is in bootloader mode"
  },
  "pin": {
    "empty": "PIN cannot be empty",
    "tooShort": "PIN must be at least {{min}} digits",
    "tooLong": "PIN cannot be longer than {{max}} digits",
    "invalid": "Invalid PIN",
    "mismatch": "PINs do not match",
    "incorrect": "Incorrect PIN. {{attempts}} attempts remaining"
  },
  "network": {
    "offline": "No internet connection",
    "timeout": "Request timed out",
    "serverError": "Server error. Please try again later"
  },
  "validation": {
    "required": "This field is required",
    "invalidAddress": "Invalid address",
    "insufficientFunds": "Insufficient funds",
    "amountTooSmall": "Amount is too small",
    "amountTooLarge": "Amount is too large"
  }
}
```

### Create `src/i18n/locales/en/device.json`
```json
{
  "status": {
    "connected": "Connected",
    "disconnected": "Disconnected",
    "busy": "Busy",
    "ready": "Ready",
    "bootloader": "Bootloader Mode",
    "initialized": "Initialized",
    "notInitialized": "Not Initialized"
  },
  "info": {
    "deviceId": "Device ID",
    "firmwareVersion": "Firmware Version",
    "bootloaderVersion": "Bootloader Version",
    "model": "Model",
    "label": "Label"
  },
  "actions": {
    "update": "Update Device",
    "updateFirmware": "Update Firmware",
    "updateBootloader": "Update Bootloader",
    "wipe": "Wipe Device",
    "recover": "Recover Device",
    "initialize": "Initialize Device"
  },
  "messages": {
    "confirmOnDevice": "Please confirm on your device",
    "followDirections": "Follow the directions on your device",
    "enterPin": "Enter your PIN on the device",
    "holdButton": "Hold the button on your KeepKey"
  }
}
```

## Step 4: Initialize i18n in Your App

### Update `src/index.tsx` or `src/main.tsx`
```typescript
import React from 'react';
import ReactDOM from 'react-dom/client';
import './index.css';
import App from './App';
import './i18n'; // Import i18n configuration

const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement
);

root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

## Step 5: Create Language Switcher Component

### Create `src/components/LanguageSwitcher.tsx`
```typescript
import React from 'react';
import { useTranslation } from 'react-i18next';
import { SUPPORTED_LANGUAGES } from '../i18n/languages';

export const LanguageSwitcher: React.FC = () => {
  const { i18n } = useTranslation();

  const handleLanguageChange = (languageCode: string) => {
    i18n.changeLanguage(languageCode);
    localStorage.setItem('preferredLanguage', languageCode);
  };

  return (
    <select
      value={i18n.language}
      onChange={(e) => handleLanguageChange(e.target.value)}
      className="px-3 py-2 border rounded-md focus:outline-none focus:ring-2"
    >
      {SUPPORTED_LANGUAGES.map((lang) => (
        <option key={lang.code} value={lang.code}>
          {lang.flag} {lang.nativeName}
        </option>
      ))}
    </select>
  );
};
```

## Step 6: Migrate Components to Use Translations

### Example: Migrating a Component

#### Before:
```typescript
const DeviceStatus: React.FC = ({ device }) => {
  return (
    <div>
      <h2>Device Status</h2>
      <p>Status: {device.connected ? 'Connected' : 'Disconnected'}</p>
      <button>Update Firmware</button>
    </div>
  );
};
```

#### After:
```typescript
import { useTranslation } from 'react-i18next';

const DeviceStatus: React.FC = ({ device }) => {
  const { t } = useTranslation(['device', 'common']);

  return (
    <div>
      <h2>{t('device:status.title')}</h2>
      <p>
        {t('common:status.label')}: {' '}
        {device.connected 
          ? t('device:status.connected') 
          : t('device:status.disconnected')}
      </p>
      <button>{t('device:actions.updateFirmware')}</button>
    </div>
  );
};
```

## Step 7: Handle Dynamic Content

### Interpolation Example:
```typescript
const PinError: React.FC<{ attempts: number }> = ({ attempts }) => {
  const { t } = useTranslation('errors');
  
  return (
    <div className="error">
      {t('pin.incorrect', { attempts })}
      {/* Output: "Incorrect PIN. 3 attempts remaining" */}
    </div>
  );
};
```

### Pluralization Example:
```typescript
const ItemCount: React.FC<{ count: number }> = ({ count }) => {
  const { t } = useTranslation('common');
  
  return (
    <div>
      {t('items.count', { count })}
      {/* Output: "1 item" or "5 items" */}
    </div>
  );
};
```

## Step 8: Create Translation Hook Utilities

### Create `src/hooks/useTypedTranslation.ts`
```typescript
import { useTranslation } from 'react-i18next';
import { TFunction } from 'i18next';

export const useTypedTranslation = (namespaces?: string | string[]) => {
  const { t, i18n, ready } = useTranslation(namespaces);
  
  return {
    t: t as TFunction,
    i18n,
    ready,
    currentLanguage: i18n.language,
    changeLanguage: (lang: string) => i18n.changeLanguage(lang),
  };
};
```

## Step 9: Testing Translations

### Create `src/i18n/__tests__/translations.test.ts`
```typescript
import i18n from '../index';
import enCommon from '../locales/en/common.json';

describe('Translation System', () => {
  it('should initialize with default language', () => {
    expect(i18n.language).toBe('en');
  });

  it('should have all required namespaces', () => {
    const namespaces = ['common', 'onboarding', 'settings', 'errors', 'device'];
    namespaces.forEach(ns => {
      expect(i18n.hasResourceBundle('en', ns)).toBe(true);
    });
  });

  it('should translate keys correctly', () => {
    expect(i18n.t('common:buttons.cancel')).toBe('Cancel');
  });

  it('should handle interpolation', () => {
    const result = i18n.t('errors:pin.incorrect', { attempts: 3 });
    expect(result).toContain('3');
  });
});
```

## Step 10: Migration Checklist

### Phase 1: Core Components
- [ ] App.tsx
- [ ] OnboardingWizard components
- [ ] SettingsDialog.tsx
- [ ] Device connection dialogs
- [ ] Error messages

### Phase 2: Main Interface
- [ ] Navigation components
- [ ] Portfolio views
- [ ] Transaction components
- [ ] Address generation

### Phase 3: Secondary Features
- [ ] Help and troubleshooting
- [ ] Logging interface
- [ ] Advanced settings
- [ ] About dialog

## Common Patterns and Best Practices

### 1. Namespace Organization
```typescript
// Use namespaces to organize translations
const { t } = useTranslation(['device', 'common']);

// Access with namespace prefix
t('device:status.connected')
t('common:buttons.save')
```

### 2. Default Values
```typescript
// Provide fallback for missing translations
t('some.key', 'Default text if key missing')
```

### 3. Component-Specific Translations
```typescript
// Create a wrapper for component-specific translations
const useComponentTranslation = () => {
  const { t } = useTranslation('myComponent');
  return {
    title: t('title'),
    description: t('description'),
    // ... other translations
  };
};
```

### 4. Lazy Loading Languages
```typescript
// Load additional languages on demand
const loadLanguage = async (lang: string) => {
  try {
    const translations = await import(`./locales/${lang}/index.ts`);
    Object.keys(translations).forEach(ns => {
      i18n.addResourceBundle(lang, ns, translations[ns]);
    });
  } catch (error) {
    console.error(`Failed to load language: ${lang}`, error);
  }
};
```

### 5. Format Dates and Numbers
```typescript
import { useTranslation } from 'react-i18next';

const FormattedDate: React.FC<{ date: Date }> = ({ date }) => {
  const { i18n } = useTranslation();
  
  const formatted = new Intl.DateTimeFormat(i18n.language, {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  }).format(date);
  
  return <span>{formatted}</span>;
};
```

## Troubleshooting

### Issue: Translations not updating
**Solution**: Ensure you're using the correct namespace and key path. Check browser console for missing translation warnings.

### Issue: TypeScript errors with translation keys
**Solution**: Update the types.ts file with your translation structure and ensure all JSON files are properly typed.

### Issue: Language not persisting on refresh
**Solution**: Verify localStorage is working and the LanguageDetector is properly configured.

### Issue: Performance issues with many translations
**Solution**: Implement lazy loading for language bundles and use React.memo for translation-heavy components.

## Performance Optimization

### 1. Bundle Splitting
```typescript
// Split translations by feature
const loadFeatureTranslations = async (feature: string) => {
  const module = await import(`./locales/en/features/${feature}.json`);
  i18n.addResourceBundle('en', feature, module.default);
};
```

### 2. Memoization
```typescript
// Memoize translated values that don't change often
const TranslatedComponent = React.memo(({ translationKey }) => {
  const { t } = useTranslation();
  return <div>{t(translationKey)}</div>;
});
```

### 3. Suspense Loading
```typescript
// Use Suspense for loading translations
const LazyTranslatedView = React.lazy(() => 
  import('./views/TranslatedView')
);

<Suspense fallback={<Loading />}>
  <LazyTranslatedView />
</Suspense>
```

## Adding New Languages

### Step-by-step process:
1. Create new locale folder: `src/i18n/locales/[lang]/`
2. Copy English JSON files as templates
3. Translate all strings
4. Add language to SUPPORTED_LANGUAGES in languages.ts
5. Import in i18n/index.ts
6. Test thoroughly

### Professional Translation Workflow:
1. Export English JSON files
2. Send to translation service
3. Import translated files
4. Review for context accuracy
5. Test in application

## Maintenance and Updates

### Adding New Translation Keys:
1. Add key to English JSON first
2. Use key in component
3. Add to other languages
4. Update TypeScript types
5. Test all languages

### Translation Key Naming Conventions:
- Use descriptive, hierarchical keys
- Group related translations
- Avoid abbreviations
- Be consistent with naming patterns

Example:
```json
{
  "settings": {
    "security": {
      "pin": {
        "change": {
          "title": "Change PIN",
          "description": "Update your device PIN",
          "button": "Change PIN"
        }
      }
    }
  }
}
```

## Conclusion

This implementation guide provides a complete roadmap for adding internationalization to the vault-v2 application. Follow the steps sequentially, test thoroughly at each phase, and maintain consistent patterns throughout the migration. The result will be a fully internationalized application ready for global users.