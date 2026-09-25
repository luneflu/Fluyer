import { writable } from 'svelte/store';
import type { Locale } from '$lib/i18n';
import { setLocale, defaultLocale } from '$lib/i18n';

function createLocaleStore() {
  const { subscribe, set, update } = writable<Locale>(defaultLocale);

  return {
    subscribe,
    set: (value: Locale) => {
      setLocale(value);
      set(value);
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('fluyer_locale', value);
      }
    },
    init: () => {
      if (typeof localStorage !== 'undefined') {
        const saved = localStorage.getItem('fluyer_locale') as Locale | null;
        if (saved) {
          setLocale(saved);
          set(saved);
        }
      }
    }
  };
}

export const locale = createLocaleStore();
