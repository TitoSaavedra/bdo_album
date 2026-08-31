import './styles/tailwind.css';
import './styles/app.scss';
import { setupI18n } from './lib/i18n/index';
import App from './app/App.svelte';
import { mount } from 'svelte';

setupI18n();

const app = mount(App, { target: document.getElementById('app')! });

export default app;
