import { addMessages, init } from 'svelte-i18n';
import es from './locales/es.json';
import en from './locales/en.json';
import { settings } from '../../features/settings/state/settings.svelte';

// Boots with whatever language Settings has stored (OS-language-detected on
// first launch, see settings.svelte.ts) — switching languages afterwards
// goes through `setLanguage`, which updates the same `locale` store `init`
// seeds here.
export function setupI18n() {
  addMessages('es', es);
  addMessages('en', en);

  init({
    fallbackLocale: 'en',
    initialLocale: settings.language,
  });
}
