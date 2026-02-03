export type TabKey = 'race' | 'tires' | 'dynamics' | 'settings' | 'debug'

export type SpeedUnit = 'kph' | 'mph'
export type TempUnit = 'c' | 'f'
export type PressureUnit = 'kpa' | 'psi'
export type FuelUnit = 'l' | 'gal'

export type UiLog = {
  at: number
  level: 'info' | 'warn' | 'error'
  message: string
}
