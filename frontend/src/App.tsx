import { useEffect, useMemo, useState } from 'react'
import UplotReact from 'uplot-react'
import 'uplot/dist/uPlot.min.css'
import './App.css'

function App() {
  const [status, setStatus] = useState('connecting')
  const [hello, setHello] = useState<string | null>(null)
  const [telemetry, setTelemetry] = useState<TelemetryState>({})
  const [chartData, setChartData] = useState<number[][]>([[], []])
  const [windowLabel, setWindowLabel] = useState<string>('no samples')
  const [chartWidth, setChartWidth] = useState(getChartWidth())
  const [udpConfig, setUdpConfig] = useState<UdpConfig | null>(null)
  const [ps5Input, setPs5Input] = useState('')
  const [detectStatus, setDetectStatus] = useState<DetectStatus>('idle')
  const [detectId, setDetectId] = useState<number | null>(null)
  const [detectIp, setDetectIp] = useState<string | null>(null)
  const wsUrl = import.meta.env.VITE_WS_URL || 'ws://127.0.0.1:10086/ws'
  const raceMode = telemetry.in_race
    ? telemetry.is_paused
      ? 'Paused'
      : 'In race'
    : 'Not in race'
  const raceModeClass = telemetry.in_race
    ? telemetry.is_paused
      ? 'race-mode paused'
      : 'race-mode in-race'
    : 'race-mode not-in-race'

  const chartOptions = useMemo(
    () => ({
      width: chartWidth,
      height: 240,
      scales: { x: { time: false } },
      series: [
        { label: 't (s)' },
        { label: 'Speed (kph)', stroke: '#0b5fff', width: 2 },
      ],
      axes: [{}, { label: 'kph' }],
    }),
    [chartWidth]
  )

  useEffect(() => {
    const socket = new WebSocket(wsUrl)
    setStatus('connecting')

    socket.onopen = () => setStatus('open')
    socket.onclose = () => setStatus('closed')
    socket.onerror = () => setStatus('error')
    socket.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as TelemetryMessage
        if (!message || typeof message.type !== 'string') {
          return
        }
        if (message.type === 'handshake_hello') {
          setHello(`hello ${message.server_version}`)
        }
        if (message.type === 'state_update') {
          setTelemetry((prev) => ({ ...prev, ...message.state }))
        }
        if (message.type === 'samples_window') {
          const xs: number[] = []
          const ys: number[] = []
          for (const sample of message.window.samples) {
            xs.push((sample.t_ms - message.window.start_ms) / 1000)
            ys.push(sample.speed_kph ?? Number.NaN)
          }
          setChartData([xs, ys])
          setWindowLabel(
            `${message.window.samples.length} samples / ${(
              (message.window.end_ms - message.window.start_ms) /
              1000
            ).toFixed(1)}s`
          )
        }
      } catch {
        return
      }
    }

    return () => {
      socket.close()
    }
  }, [wsUrl])

  useEffect(() => {
    fetch('/config/udp')
      .then((res) => res.json())
      .then((data: UdpConfig) => {
        setUdpConfig(data)
        setPs5Input(data.ps5_ip ?? '')
      })
      .catch(() => {
        setUdpConfig(null)
      })
  }, [])

  useEffect(() => {
    if (detectId === null) {
      return
    }
    let cancelled = false
    const timer = window.setInterval(async () => {
      try {
        const res = await fetch(`/config/udp/auto-detect/${detectId}`)
        const data = (await res.json()) as DetectStatusResponse
        if (cancelled) {
          return
        }
        setDetectStatus(data.status)
        setDetectIp(data.ps5_ip ?? null)
        if (data.status !== 'pending') {
          window.clearInterval(timer)
          setDetectId(null)
          const cfg = await fetch('/config/udp').then((r) => r.json())
          setUdpConfig(cfg)
          setPs5Input(cfg.ps5_ip ?? '')
        }
      } catch {
        if (!cancelled) {
          setDetectStatus('error')
          setDetectId(null)
          window.clearInterval(timer)
        }
      }
    }, 1000)
    return () => {
      cancelled = true
      window.clearInterval(timer)
    }
  }, [detectId])

  useEffect(() => {
    const onResize = () => setChartWidth(getChartWidth())
    window.addEventListener('resize', onResize)
    return () => window.removeEventListener('resize', onResize)
  }, [])

  return (
    <>
      <div className="app">
        <header className="header">
          <div>
            <h1>GT7 LapLab</h1>
            <p className="subtitle">Live telemetry preview</p>
          </div>
          <div className="status">
            <span>WS</span>
            <strong>{status}</strong>
          </div>
        </header>

        <section className="panel">
          <div className="panel-header">
            <h2>Connection</h2>
            <span className="mono">{wsUrl}</span>
          </div>
          <div className="panel-body">
            <div className="kv">
              <span>Handshake</span>
              <strong>{hello ?? 'waiting'}</strong>
            </div>
          </div>
        </section>

        <section className="panel">
          <div className="panel-header">
            <h2>PS5 Connection</h2>
            <span className="mono">
              UDP bind {udpConfig?.bind_addr ?? 'unknown'}
            </span>
          </div>
          <div className="panel-body">
            <div className="ps5-form">
              <label>
                PS5 IP
                <input
                  value={ps5Input}
                  onChange={(event) => setPs5Input(event.target.value)}
                  placeholder="192.168.1.10"
                />
              </label>
              <div className="ps5-actions">
                <button
                  type="button"
                  onClick={async () => {
                    if (!udpConfig) {
                      return
                    }
                    const payload = {
                      bind_addr: udpConfig.bind_addr,
                      ps5_ip: ps5Input.trim() === '' ? null : ps5Input.trim(),
                    }
                    const res = await fetch('/config/udp', {
                      method: 'POST',
                      headers: { 'Content-Type': 'application/json' },
                      body: JSON.stringify(payload),
                    })
                    if (res.ok) {
                      const data = (await res.json()) as UdpConfig
                      setUdpConfig(data)
                      setPs5Input(data.ps5_ip ?? '')
                    }
                  }}
                >
                  Save
                </button>
                <button
                  type="button"
                  className="ghost"
                  onClick={async () => {
                    setDetectStatus('pending')
                    setDetectIp(null)
                    const res = await fetch('/config/udp/auto-detect', {
                      method: 'POST',
                    })
                    if (!res.ok) {
                      setDetectStatus('error')
                      return
                    }
                    const data = (await res.json()) as DetectStartResponse
                    setDetectStatus(data.status)
                    setDetectId(data.id)
                  }}
                >
                  Auto-detect
                </button>
              </div>
            </div>
            <div className="kv">
              <span>Status</span>
              <strong>{detectStatus}</strong>
            </div>
            <div className="kv">
              <span>Detected IP</span>
              <strong>{detectIp ?? '-'}</strong>
            </div>
          </div>
        </section>

        <section className="panel">
          <div className="panel-header">
            <h2>Live Gauges</h2>
            <div className="panel-meta">
              <span className={raceModeClass}>{raceMode}</span>
              <span className="mono">20 Hz state</span>
            </div>
          </div>
          <div className="gauges">
            <Gauge
              label="Speed"
              value={telemetry.speed_kph}
              unit="kph"
              max={350}
              precision={1}
            />
            <Gauge label="RPM" value={telemetry.rpm} unit="rpm" max={9000} />
            <Gear value={telemetry.gear} />
            <Gauge
              label="Throttle"
              value={toPercent(telemetry.throttle)}
              unit="%"
              max={100}
            />
            <Gauge
              label="Brake"
              value={toPercent(telemetry.brake)}
              unit="%"
              max={100}
            />
          </div>
        </section>

        <section className="panel">
          <div className="panel-header">
            <h2>Speed Trace</h2>
            <span className="mono">{windowLabel}</span>
          </div>
          <div className="panel-body chart">
            <UplotReact options={chartOptions} data={chartData} />
          </div>
        </section>
      </div>
    </>
  )
}

type TelemetryState = {
  speed_kph?: number
  rpm?: number
  gear?: number
  throttle?: number
  brake?: number
  in_race?: boolean
  is_paused?: boolean
}

type UdpConfig = {
  bind_addr: string
  ps5_ip: string | null
}

type DetectStatus = 'idle' | 'pending' | 'found' | 'timeout' | 'error' | 'cancelled'

type DetectStartResponse = {
  id: number
  status: DetectStatus
  timeout_ms: number
}

type DetectStatusResponse = {
  id: number
  status: DetectStatus
  ps5_ip: string | null
}

type Sample = {
  t_ms: number
  speed_kph?: number
  rpm?: number
  throttle?: number
  brake?: number
}

type HandshakeHello = {
  type: 'handshake_hello'
  server_version: string
}

type StateUpdate = {
  type: 'state_update'
  state: TelemetryState
}

type SamplesWindow = {
  type: 'samples_window'
  window: {
    start_ms: number
    end_ms: number
    stride_ms: number
    samples: Sample[]
  }
}

type TelemetryMessage = HandshakeHello | StateUpdate | SamplesWindow

function toPercent(value?: number) {
  if (value === undefined) {
    return undefined
  }
  return value * 100
}

function getChartWidth() {
  if (typeof window === 'undefined') {
    return 720
  }
  return Math.min(720, Math.max(320, window.innerWidth - 80))
}

function Gauge({
  label,
  value,
  unit,
  max,
  precision = 0,
}: {
  label: string
  value?: number
  unit: string
  max: number
  precision?: number
}) {
  const safeValue = value ?? 0
  const pct = Math.min(Math.max(safeValue / max, 0), 1)
  const display =
    value === undefined ? '--' : safeValue.toFixed(precision).toString()

  return (
    <div className="gauge">
      <div className="gauge-header">
        <span>{label}</span>
        <strong>
          {display} <em>{unit}</em>
        </strong>
      </div>
      <div className="gauge-bar">
        <div className="gauge-fill" style={{ width: `${pct * 100}%` }} />
      </div>
    </div>
  )
}

function Gear({ value }: { value?: number }) {
  let display = '--'
  if (value === -1) {
    display = 'R'
  } else if (value !== undefined) {
    display = value.toString()
  }
  return (
    <div className="gauge gear">
      <div className="gauge-header">
        <span>Gear</span>
        <strong className="gear-value">{display}</strong>
      </div>
      <div className="gear-pill">Drive</div>
    </div>
  )
}

export default App
