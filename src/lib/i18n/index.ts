import en from './locales/en.json';
import it from './locales/it.json';

export const locales = { en, it } as const;
export type Locale = keyof typeof locales;
export type TranslationKey = keyof typeof en;

export const defaultLocale: Locale = 'en';
export const supportedLocales: Locale[] = ['en', 'it'];

let current: Locale = defaultLocale;

export function setLocale(l: Locale) {
  if (supportedLocales.includes(l)) {
    current = l;
  }
}

export function getLocale(): Locale {
  return current;
}

export function t(key: TranslationKey): string {
  const dict = locales[current]?? locales[defaultLocale];
  return (dict[key] as string)?? (locales[defaultLocale][key] as string)?? key;
}

export function getAllKeys(): TranslationKey[] {
  return Object.keys(locales[defaultLocale]) as TranslationKey[];
}
