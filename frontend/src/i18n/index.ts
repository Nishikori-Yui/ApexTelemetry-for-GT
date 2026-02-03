import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'

import en from './en.json'
import zhCN from './zh-CN.json'
import zhTW from './zh-TW.json'
import ja from './ja.json'

export const normalizeLang = (value: string | null): string => {
  if (!value) {
    return 'en'
  }
  const lower = value.toLowerCase()
  if (lower.startsWith('zh-tw') || lower.startsWith('zh-hant')) {
    return 'zh-TW'
  }
  if (lower.startsWith('zh-cn') || lower.startsWith('zh-hans') || lower === 'zh') {
    return 'zh-CN'
  }
  if (lower.startsWith('ja')) {
    return 'ja'
  }
  if (lower.startsWith('en')) {
    return 'en'
  }
  return value
}

const storedLang =
  typeof window !== 'undefined'
    ? window.localStorage.getItem('apextelemetry.lang')
    : null
const initialLang = normalizeLang(storedLang)

i18n.use(initReactI18next).init({
  resources: {
    en: { translation: en },
    'zh-CN': { translation: zhCN },
    'zh-TW': { translation: zhTW },
    ja: { translation: ja },
  },
  lng: initialLang,
  fallbackLng: 'en',
  supportedLngs: ['en', 'zh-CN', 'zh-TW', 'ja'],
  interpolation: { escapeValue: false },
})

export default i18n
