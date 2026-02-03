import type { Sample, TelemetryState } from './telemetry'

export type HandshakeHello = {
  type: 'handshake_hello'
  server_version: string
}

export type StateUpdate = {
  type: 'state_update'
  state: TelemetryState
}

export type SamplesWindow = {
  type: 'samples_window'
  window: {
    start_ms: number
    end_ms: number
    stride_ms: number
    samples: Sample[]
  }
}

export type TelemetryMessage = HandshakeHello | StateUpdate | SamplesWindow
