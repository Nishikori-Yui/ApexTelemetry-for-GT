import init, { decode_demo_bin } from '../wasm/pkg/telemetry_wasm.js'
import type { DemoFrame } from './types'

let wasmReady: Promise<unknown> | null = null

const ensureWasmReady = async () => {
  if (!wasmReady) {
    wasmReady = init()
  }
  await wasmReady
}

export const decodeDemoBin = async (buffer: ArrayBuffer, trackId?: number | null, carId?: number | null) => {
  await ensureWasmReady()
  const frames = decode_demo_bin(new Uint8Array(buffer), trackId ?? null, carId ?? null) as DemoFrame[]
  return frames
}
