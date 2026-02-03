export const UNIT_KEYS = {
  speed: 'apextelemetry.units.speed',
  temp: 'apextelemetry.units.temp',
  pressure: 'apextelemetry.units.pressure',
  fuel: 'apextelemetry.units.fuel',
} as const

export const KPH_TO_MPH = 0.621371
export const KPA_TO_PSI = 0.1450377377
export const L_TO_GAL = 0.264172

export function getStoredUnit<T extends string>(
  key: string,
  allowed: readonly T[],
  fallback: T,
) {
  if (typeof window === 'undefined') {
    return fallback
  }
  const stored = window.localStorage.getItem(key)
  if (stored && allowed.includes(stored as T)) {
    return stored as T
  }
  return fallback
}

export function toMph(kph: number) {
  return kph * KPH_TO_MPH
}

export function toKph(mph: number) {
  return mph / KPH_TO_MPH
}

export function toFahrenheit(celsius: number) {
  return (celsius * 9) / 5 + 32
}

export function toCelsius(fahrenheit: number) {
  return ((fahrenheit - 32) * 5) / 9
}

export function toPsi(kpa: number) {
  return kpa * KPA_TO_PSI
}

export function toKpa(psi: number) {
  return psi / KPA_TO_PSI
}

export function toGallons(liters: number) {
  return liters * L_TO_GAL
}

export function toLiters(gallons: number) {
  return gallons / L_TO_GAL
}
