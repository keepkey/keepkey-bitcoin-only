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