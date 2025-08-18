import 'react-i18next';
import type common from './locales/en/common.json';
import type onboarding from './locales/en/onboarding.json';
import type settings from './locales/en/settings.json';
import type errors from './locales/en/errors.json';
import type device from './locales/en/device.json';
import type wallet from './locales/en/wallet.json';
import type transaction from './locales/en/transaction.json';

declare module 'react-i18next' {
  interface CustomTypeOptions {
    defaultNS: 'common';
    resources: {
      common: typeof common;
      onboarding: typeof onboarding;
      settings: typeof settings;
      errors: typeof errors;
      device: typeof device;
      wallet: typeof wallet;
      transaction: typeof transaction;
    };
  }
}