import { createI18n } from 'vue-i18n';
import zhCN from '@/locales/zh-CN.json';
import en from '@/locales/en.json';

const i18n = createI18n({
  legacy: false, // Composition API mode (required for vue-i18n v11)
  locale: 'zh-CN',
  fallbackLocale: 'en',
  messages: {
    'zh-CN': zhCN,
    en,
  },
});

export default i18n;
