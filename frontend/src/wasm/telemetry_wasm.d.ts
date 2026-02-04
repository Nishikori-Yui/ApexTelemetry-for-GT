declare module '../wasm/pkg/telemetry_wasm.js' {
  export default function init(
    input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module,
  ): Promise<void>
  export function decode_demo_bin(
    data: Uint8Array,
    fixedTrackId?: number | null,
    fixedCarId?: number | null,
  ): unknown
}
