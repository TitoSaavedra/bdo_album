import { addMessages, init } from 'svelte-i18n';
import es from './locales/es.json';
import en from './locales/en.json';

// English-only for now — no locale detection, no in-app switcher.
// es.json only has the DB error keys so far; not expanded for the rest of
// the UI yet since nothing switches to it.
export function setupI18n() {
  addMessages('es', es);
  addMessages('en', en);

  init({
    fallbackLocale: 'en',
    initialLocale: 'en',
  });
}
