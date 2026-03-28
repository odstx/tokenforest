import { derived, get } from 'svelte/store';
import { dictionary, locale, addMessages, init } from 'svelte-i18n';
import { browser } from '$app/environment';
import en from './translations/en.json';
import zh from './translations/zh.json';

addMessages('en', en);
addMessages('zh', zh);

const defaultLocale = 'en';

function getInitialLocale(): string {
  if (!browser) return defaultLocale;
  
  const saved = localStorage.getItem('locale');
  if (saved && (saved === 'en' || saved === 'zh')) {
    return saved;
  }
  
  const browserLang = navigator.language.toLowerCase();
  if (browserLang.startsWith('zh')) {
    return 'zh';
  }
  
  return defaultLocale;
}

export function setupI18n(): void {
  init({
    fallbackLocale: defaultLocale,
    initialLocale: getInitialLocale(),
  });
}

export function setLocale(newLocale: string): void {
  if (browser) {
    localStorage.setItem('locale', newLocale);
  }
  locale.set(newLocale);
}

export const currentLocale = derived(locale, ($locale) => $locale || defaultLocale);

export function t(key: string, params?: Record<string, string>): string {
  const currentLocaleValue = get(locale) || defaultLocale;
  const dict = get(dictionary);
  const messages = dict[currentLocaleValue];
  
  if (!messages) return key;
  
  let message: string = key;
  const currentMsg = messages[key];
  if (typeof currentMsg === 'string') {
    message = currentMsg;
  } else {
    const fallbackDict = dict[defaultLocale];
    const fallbackMsg = fallbackDict?.[key];
    if (typeof fallbackMsg === 'string') {
      message = fallbackMsg;
    }
  }
  
  if (params && message !== key) {
    Object.entries(params).forEach(([k, v]) => {
      message = message.replace(new RegExp(`{${k}}`, 'g'), v);
    });
  }
  
  return message;
}
