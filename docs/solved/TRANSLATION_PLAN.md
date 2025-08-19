# KeepKey Vault-v2 Translation Implementation Plan

## Executive Summary

The vault-v2 application currently contains **450-500+ hardcoded English strings** that need internationalization support. This document outlines a comprehensive plan to implement a translation system using industry-standard i18n practices.

## Current State Analysis

### Statistics
- **Total Hardcoded Strings**: ~450-500
- **Affected Files**: 80+ TypeScript/TSX files
- **Primary Categories**: UI labels, error messages, instructions, status messages
- **Current Language**: English only

### High-Impact Areas
1. **Onboarding Wizard** - First user experience (30+ strings)
2. **Setup Wizard** - Device initialization (40+ strings)
3. **Settings Dialog** - User preferences (45+ strings)
4. **Error Messages** - Critical feedback (60+ strings)
5. **Device Interactions** - Hardware communication (35+ strings)

## Translation Framework Selection

### Recommended: react-i18next
**Reasons for Selection**:
- React-specific optimizations
- Built-in React hooks (useTranslation)
- Lazy loading support for language bundles
- Pluralization and formatting support
- TypeScript support with type safety
- Active community and ecosystem
- Built-in fallback language support

### Alternative Considered: react-intl (FormatJS)
- More complex API
- Heavier bundle size
- Better for ICU message format needs

## Implementation Phases

### Phase 1: Foundation Setup (Week 1)
- [ ] Install and configure react-i18next
- [ ] Set up translation file structure
- [ ] Create base language files (en-US)
- [ ] Implement language detection logic
- [ ] Add language switcher component
- [ ] Set up TypeScript types for translations

### Phase 2: Critical Path Translation (Week 2)
- [ ] Translate Onboarding Wizard
- [ ] Translate Setup Wizard
- [ ] Translate error messages
- [ ] Translate device connection flows
- [ ] Implement dynamic string interpolation

### Phase 3: Main Interface (Week 3)
- [ ] Translate Settings Dialog
- [ ] Translate main navigation
- [ ] Translate portfolio views
- [ ] Translate transaction interfaces
- [ ] Handle pluralization cases

### Phase 4: Secondary Features (Week 4)
- [ ] Translate help and troubleshooting
- [ ] Translate tooltips and hints
- [ ] Translate logging interfaces
- [ ] Translate remaining components
- [ ] Add missing translation keys

### Phase 5: Testing & Refinement (Week 5)
- [ ] Comprehensive translation testing
- [ ] RTL language support verification
- [ ] Performance optimization
- [ ] Bundle size optimization
- [ ] User acceptance testing

## Technical Architecture

### File Structure
```
vault-v2/
├── src/
│   ├── i18n/
│   │   ├── index.ts           # i18n configuration
│   │   ├── languages.ts       # Supported languages list
│   │   ├── types.ts          # TypeScript definitions
│   │   └── locales/
│   │       ├── en/
│   │       │   ├── common.json
│   │       │   ├── onboarding.json
│   │       │   ├── settings.json
│   │       │   ├── errors.json
│   │       │   └── device.json
│   │       ├── es/
│   │       ├── fr/
│   │       ├── de/
│   │       ├── ja/
│   │       └── zh/
```

### Translation Key Structure
```typescript
{
  "common": {
    "buttons": {
      "cancel": "Cancel",
      "confirm": "Confirm",
      "save": "Save",
      "close": "Close"
    },
    "status": {
      "loading": "Loading...",
      "error": "Error",
      "success": "Success"
    }
  },
  "onboarding": {
    "language": {
      "title": "Select Your Language",
      "description": "Choose your preferred language"
    }
  },
  "errors": {
    "device": {
      "notConnected": "No KeepKey device connected",
      "connectionFailed": "Failed to connect to device"
    },
    "pin": {
      "empty": "PIN cannot be empty",
      "tooLong": "PIN cannot be longer than {{max}} digits"
    }
  }
}
```

### Component Integration Pattern
```typescript
// Before
const MyComponent = () => {
  return <div>Hello World</div>;
};

// After
import { useTranslation } from 'react-i18next';

const MyComponent = () => {
  const { t } = useTranslation();
  return <div>{t('common.greeting')}</div>;
};
```

## Language Support Priority

### Tier 1 (Launch)
- **en-US** - English (United States) - Base language
- **es** - Spanish
- **zh-CN** - Simplified Chinese
- **ja** - Japanese

### Tier 2 (Post-launch)
- **de** - German
- **fr** - French
- **ko** - Korean
- **pt-BR** - Portuguese (Brazil)

### Tier 3 (Future)
- **ru** - Russian
- **it** - Italian
- **ar** - Arabic (RTL support required)
- **hi** - Hindi

## Special Considerations

### 1. Device Communication
- Hardware messages must remain in English for firmware compatibility
- Display user-friendly translations while maintaining English protocol

### 2. Dynamic Content
```typescript
// Handle variables in translations
t('device.status', { deviceId: device.id })
// "Device {{deviceId}} is connected"
```

### 3. Pluralization
```typescript
// Handle plural forms
t('items.count', { count: items.length })
// "You have {{count}} item" / "You have {{count}} items"
```

### 4. Date/Time Formatting
- Use i18next formatting plugins
- Respect locale-specific date/time formats

### 5. Number Formatting
- Currency display based on locale
- Decimal separators (. vs ,)

## Performance Considerations

### Bundle Size Management
- Lazy load language bundles
- Only load active language
- Code-split translation files
- Target: <50KB per language bundle

### Loading Strategy
```typescript
// Lazy load languages
const loadLanguage = async (lang: string) => {
  const resources = await import(`./locales/${lang}/index.ts`);
  i18n.addResourceBundle(lang, 'translation', resources.default);
};
```

## Quality Assurance

### Translation Testing Checklist
- [ ] All strings properly externalized
- [ ] No hardcoded strings remaining
- [ ] Proper fallback for missing translations
- [ ] Variable interpolation working
- [ ] Pluralization rules correct
- [ ] RTL languages display correctly
- [ ] Text truncation handled gracefully
- [ ] Date/time formats locale-appropriate

### Automated Testing
```typescript
// Test for missing translations
describe('Translations', () => {
  it('should have all required keys', () => {
    const languages = ['en', 'es', 'zh', 'ja'];
    languages.forEach(lang => {
      expect(hasAllKeys(lang)).toBe(true);
    });
  });
});
```

## Migration Strategy

### Step 1: Non-breaking Introduction
- Add i18n alongside existing strings
- Gradually replace hardcoded strings
- Maintain backward compatibility

### Step 2: Progressive Migration
- Start with high-impact areas
- Component-by-component migration
- Maintain working application throughout

### Step 3: Cleanup
- Remove all hardcoded strings
- Optimize bundle sizes
- Performance testing

## Success Metrics

### Technical Metrics
- 100% string externalization
- <50KB per language bundle
- <100ms language switch time
- Zero translation-related errors

### User Metrics
- Language preference retention
- Reduced support tickets for non-English users
- Increased adoption in target markets
- User satisfaction scores by language

## Risk Mitigation

### Identified Risks
1. **Performance Impact**: Mitigate with lazy loading
2. **Bundle Size Growth**: Use code splitting
3. **Translation Quality**: Professional translation services
4. **Maintenance Burden**: Automated tooling and processes
5. **Testing Complexity**: Comprehensive test suite

## Maintenance Plan

### Translation Workflow
1. Developers add keys in English
2. Keys extracted to translation files
3. Send for professional translation
4. Review and integrate translations
5. Test in application context

### Tooling
- Use translation management platform (Crowdin/Lokalise)
- Automated extraction of new keys
- Translation memory for consistency
- Version control for translation files

## Timeline and Resources

### Timeline: 5 Weeks
- Week 1: Foundation setup
- Week 2: Critical path translation
- Week 3: Main interface
- Week 4: Secondary features
- Week 5: Testing and refinement

### Resources Required
- 1 Senior Developer (full-time)
- 1 QA Engineer (50%)
- Translation services for 4 languages
- Translation management platform subscription

## Conclusion

Implementing internationalization in vault-v2 is a significant undertaking with ~500 strings to translate. Using react-i18next and following this phased approach will provide a robust, maintainable translation system that can scale to support multiple languages and regions. The investment will significantly improve accessibility and user experience for non-English speaking users.