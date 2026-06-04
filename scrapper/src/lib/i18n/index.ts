import { addMessages, init, getLocaleFromNavigator } from 'svelte-i18n';
import es from './locales/es.json';
import en from './locales/en.json';

export function setupI18n() {
  addMessages('es', es);
  addMessages('en', en);

  init({
    fallbackLocale: 'es',
    initialLocale: getLocaleFromNavigator() ?? 'es',
  });
}
