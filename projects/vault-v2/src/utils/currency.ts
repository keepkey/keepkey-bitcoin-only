// Currency and number formatting utilities

export interface CurrencyConfig {
  code: string;
  symbol: string;
  name: string;
  rate?: number; // Exchange rate to USD
}

export const SUPPORTED_CURRENCIES: CurrencyConfig[] = [
  { code: 'USD', symbol: '$', name: 'US Dollar' },
  { code: 'EUR', symbol: '€', name: 'Euro' },
  { code: 'GBP', symbol: '£', name: 'British Pound' },
  { code: 'JPY', symbol: '¥', name: 'Japanese Yen' },
  { code: 'CNY', symbol: '¥', name: 'Chinese Yuan' },
];

export const NUMBER_FORMATS = {
  'en-US': { code: '1,000.00', locale: 'en-US', groupSeparator: ',', decimalSeparator: '.' },
  'de-DE': { code: '1.000,00', locale: 'de-DE', groupSeparator: '.', decimalSeparator: ',' },
  'fr-FR': { code: '1 000,00', locale: 'fr-FR', groupSeparator: ' ', decimalSeparator: ',' },
  'no-decimal': { code: '1,000', locale: 'en-US', groupSeparator: ',', decimalSeparator: '.', decimals: 0 },
};

export const getStoredCurrency = (): string => {
  return localStorage.getItem('preferredCurrency') || 'USD';
};

export const getStoredNumberFormat = (): string => {
  return localStorage.getItem('numberFormat') || '1,000.00';
};

export const getCurrencyConfig = (currencyCode: string): CurrencyConfig => {
  return SUPPORTED_CURRENCIES.find(c => c.code === currencyCode) || SUPPORTED_CURRENCIES[0];
};

export const getNumberFormatConfig = (formatCode: string) => {
  switch (formatCode) {
    case '1.000,00':
      return NUMBER_FORMATS['de-DE'];
    case '1 000,00':
      return NUMBER_FORMATS['fr-FR'];
    case '1,000':
      return NUMBER_FORMATS['no-decimal'];
    default:
      return NUMBER_FORMATS['en-US'];
  }
};

export const formatCurrency = (
  amount: number,
  currencyCode?: string,
  formatCode?: string,
  showCurrency: boolean = true
): string => {
  const currency = currencyCode || getStoredCurrency();
  const format = formatCode || getStoredNumberFormat();
  const currencyConfig = getCurrencyConfig(currency);
  const numberConfig = getNumberFormatConfig(format);

  try {
    const formatter = new Intl.NumberFormat(numberConfig.locale, {
      style: showCurrency ? 'currency' : 'decimal',
      currency: currency,
      currencyDisplay: 'symbol',
      minimumFractionDigits: numberConfig.decimals ?? 2,
      maximumFractionDigits: numberConfig.decimals ?? 2,
    });

    return formatter.format(amount);
  } catch (error) {
    // Fallback formatting if Intl.NumberFormat fails
    const decimals = numberConfig.decimals ?? 2;
    const fixed = amount.toFixed(decimals);
    const parts = fixed.split('.');
    
    // Add thousand separators
    parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, numberConfig.groupSeparator);
    
    // Join with decimal separator if needed
    const formatted = decimals > 0 ? parts.join(numberConfig.decimalSeparator) : parts[0];
    
    return showCurrency ? `${currencyConfig.symbol}${formatted}` : formatted;
  }
};

export const formatNumber = (
  value: number,
  formatCode?: string,
  decimals?: number
): string => {
  const format = formatCode || getStoredNumberFormat();
  const numberConfig = getNumberFormatConfig(format);
  const finalDecimals = decimals ?? (numberConfig.decimals ?? 2);

  try {
    const formatter = new Intl.NumberFormat(numberConfig.locale, {
      minimumFractionDigits: finalDecimals,
      maximumFractionDigits: finalDecimals,
    });

    return formatter.format(value);
  } catch (error) {
    // Fallback formatting
    const fixed = value.toFixed(finalDecimals);
    const parts = fixed.split('.');
    
    parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, numberConfig.groupSeparator);
    
    return finalDecimals > 0 ? parts.join(numberConfig.decimalSeparator) : parts[0];
  }
};

export const parseCurrencyInput = (input: string, formatCode?: string): number => {
  const format = formatCode || getStoredNumberFormat();
  const numberConfig = getNumberFormatConfig(format);
  
  // Remove currency symbols and spaces
  let cleaned = input.replace(/[$€£¥₹₽]/g, '').trim();
  
  // Handle different decimal separators
  if (numberConfig.decimalSeparator === ',') {
    // European format: replace last comma with dot and remove other separators
    const lastCommaIndex = cleaned.lastIndexOf(',');
    if (lastCommaIndex > -1) {
      cleaned = cleaned.substring(0, lastCommaIndex).replace(/[,.]/g, '') + 
                '.' + cleaned.substring(lastCommaIndex + 1);
    } else {
      cleaned = cleaned.replace(/[,.]/g, '');
    }
  } else {
    // US format: remove thousand separators (commas) but keep decimal dot
    const lastDotIndex = cleaned.lastIndexOf('.');
    if (lastDotIndex > -1 && cleaned.length - lastDotIndex <= 3) {
      // Last dot is likely decimal
      cleaned = cleaned.substring(0, lastDotIndex).replace(/[,.]/g, '') + 
                cleaned.substring(lastDotIndex);
    } else {
      // No decimal point or dot is thousand separator
      cleaned = cleaned.replace(/[,.]/g, '');
    }
  }
  
  const parsed = parseFloat(cleaned);
  return isNaN(parsed) ? 0 : parsed;
};

// React hook for currency formatting
export const useCurrencyFormatter = () => {
  const currency = getStoredCurrency();
  const format = getStoredNumberFormat();
  
  return {
    formatCurrency: (amount: number, showCurrency = true) => 
      formatCurrency(amount, currency, format, showCurrency),
    formatNumber: (value: number, decimals?: number) => 
      formatNumber(value, format, decimals),
    parseCurrency: (input: string) => 
      parseCurrencyInput(input, format),
    currency,
    format,
    currencyConfig: getCurrencyConfig(currency),
    numberConfig: getNumberFormatConfig(format),
  };
};