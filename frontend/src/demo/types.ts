import type {
  DebugTelemetryResponse,
  MetaCarResponse,
  MetaTrackResponse,
  TelemetryState,
  TrackGeometrySvg,
} from '../types'

export type DemoFrame = {
  t_ms: number
  state: TelemetryState
}

export type DemoMeta = {
  car?: MetaCarResponse
  track?: MetaTrackResponse
  geometry?: TrackGeometrySvg
  debug?: DebugTelemetryResponse
}
