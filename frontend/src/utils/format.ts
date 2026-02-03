export type NumberFormats = {
  speed: Intl.NumberFormat
  raw: Intl.NumberFormat
  ratio: Intl.NumberFormat
  fuel: Intl.NumberFormat
  fuelPct: Intl.NumberFormat
  rpm: Intl.NumberFormat
  percent: Intl.NumberFormat
  int: Intl.NumberFormat
}

export function createNumberFormats(locale: string): NumberFormats {
  return {
    speed: new Intl.NumberFormat(locale, {
      minimumFractionDigits: 1,
      maximumFractionDigits: 1,
    }),
    raw: new Intl.NumberFormat(locale, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 3,
    }),
    ratio: new Intl.NumberFormat(locale, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 3,
    }),
    fuel: new Intl.NumberFormat(locale, {
      minimumFractionDigits: 1,
      maximumFractionDigits: 1,
    }),
    fuelPct: new Intl.NumberFormat(locale, {
      minimumFractionDigits: 1,
      maximumFractionDigits: 1,
    }),
    rpm: new Intl.NumberFormat(locale, {
      maximumFractionDigits: 0,
    }),
    percent: new Intl.NumberFormat(locale, {
      maximumFractionDigits: 0,
    }),
    int: new Intl.NumberFormat(locale, {
      maximumFractionDigits: 0,
    }),
  }
}

export function formatHMS(ms?: number) {
  if (ms === undefined || ms <= 0) {
    return '—'
  }
  const totalSeconds = Math.floor(ms / 1000)
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60
  const hh = String(hours).padStart(2, '0')
  const mm = String(minutes).padStart(2, '0')
  const ss = String(seconds).padStart(2, '0')
  return `${hh}:${mm}:${ss}`
}

export function formatLap(ms?: number) {
  if (ms === undefined || ms <= 0) {
    return '—'
  }
  const minutes = Math.floor(ms / 60000)
  const seconds = Math.floor((ms % 60000) / 1000)
  const millis = Math.floor(ms % 1000)
  const ss = String(seconds).padStart(2, '0')
  const mmm = String(millis).padStart(3, '0')
  return `${minutes}:${ss}.${mmm}`
}
