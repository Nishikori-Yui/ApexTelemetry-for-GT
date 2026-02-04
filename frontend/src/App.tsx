import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import './App.css'
import { decodeDemoBin } from './demo/wasm'
import type { DemoFrame, DemoMeta } from './demo/types'
import { normalizeLang } from './i18n'
import { DebugTab } from './tabs/debug/DebugTab'
import { DynamicsTab } from './tabs/dynamics/DynamicsTab'
import { RaceTab } from './tabs/race/RaceTab'
import { SettingsTab } from './tabs/settings/SettingsTab'
import { TiresTab } from './tabs/tires/TiresTab'
import type {
  DebugTelemetryResponse,
  DemoStatusResponse,
  DetectStartResponse,
  DetectStatus,
  DetectStatusResponse,
  MetaCarResponse,
  MetaTrackResponse,
  Sample,
  SpeedUnit,
  TempUnit,
  PressureUnit,
  FuelUnit,
  TabKey,
  TelemetryMessage,
  TelemetryState,
  TrackGeometrySvg,
  UiLog,
  UdpConfig,
} from './types'
import { createNumberFormats } from './utils/format'
import { getStoredUnit, UNIT_KEYS } from './utils/units'

const emptyDebugData = (timestampMs: number): DebugTelemetryResponse => ({
  timestamp_ms: timestampMs,
  session: {},
  powertrain: {},
  fluids: {},
  tyres: {},
  wheels: {},
  chassis: {},
  gears: {},
  dynamics: {},
  flags: {},
  raw: {},
})

const buildDebugFromState = (state: TelemetryState, timestampMs: number): DebugTelemetryResponse => ({
  timestamp_ms: timestampMs,
  session: {
    in_race: state.in_race ?? null,
    is_paused: state.is_paused ?? null,
    packet_id: state.packet_id ?? null,
    time_on_track_ms: state.time_on_track_ms ?? null,
    current_lap: state.current_lap ?? null,
    total_laps: state.total_laps ?? null,
    best_lap_ms: state.best_lap_ms ?? null,
    last_lap_ms: state.last_lap_ms ?? null,
    current_position: state.current_position ?? null,
    total_positions: state.total_positions ?? null,
    car_id: state.car_id ?? null,
    track_id: state.track_id ?? null,
  },
  powertrain: {
    speed_kph: state.speed_kph ?? null,
    rpm: state.rpm ?? null,
    gear: state.gear ?? null,
    throttle: state.throttle ?? null,
    brake: state.brake ?? null,
    boost_kpa: state.boost_kpa ?? null,
  },
  fluids: {
    fuel_l: state.fuel_l ?? null,
    fuel_capacity_l: state.fuel_capacity_l ?? null,
  },
  tyres: {},
  wheels: {},
  chassis: {},
  gears: {},
  dynamics: {
    pos_x: state.pos_x ?? null,
    pos_y: state.pos_y ?? null,
    pos_z: state.pos_z ?? null,
    vel_x: state.vel_x ?? null,
    vel_y: state.vel_y ?? null,
    vel_z: state.vel_z ?? null,
    rotation_yaw: state.rotation_yaw ?? null,
  },
  flags: {},
  raw: {},
})

function App() {
  const { t, i18n } = useTranslation()
  const [activeTab, setActiveTab] = useState<TabKey>('race')
  const [status, setStatus] = useState('connecting')
  const [telemetry, setTelemetry] = useState<TelemetryState>({})
  const [lastTelemetryAt, setLastTelemetryAt] = useState<number | null>(null)
  const samplesWindowRef = useRef<{
    samples: Sample[]
    range: { start_ms: number; end_ms: number } | null
    info: { count: number; seconds: number } | null
  }>({
    samples: [],
    range: null,
    info: null,
  })
  const [udpConfig, setUdpConfig] = useState<UdpConfig | null>(null)
  const [ps5Input, setPs5Input] = useState('')
  const [detectStatus, setDetectStatus] = useState<DetectStatus>('idle')
  const [detectId, setDetectId] = useState<number | null>(null)
  const [detectIp, setDetectIp] = useState<string | null>(null)
  const [demoActive, setDemoActive] = useState(false)
  const [demoPath, setDemoPath] = useState<string | null>(null)
  const [demoPending, setDemoPending] = useState(false)
  const [demoError, setDemoError] = useState<string | null>(null)
  const [uiDebugEnabled, setUiDebugEnabled] = useState(false)
  const [uiLogs, setUiLogs] = useState<UiLog[]>([])
  const [tickNow, setTickNow] = useState(0)
  const [debugData, setDebugData] = useState<DebugTelemetryResponse | null>(null)
  const [debugCopied, setDebugCopied] = useState(false)
  const [debugRawCopied, setDebugRawCopied] = useState(false)
  const [metaCar, setMetaCar] = useState<MetaCarResponse | null>(null)
  const [metaTrack, setMetaTrack] = useState<MetaTrackResponse | null>(null)
  const [trackGeometry, setTrackGeometry] = useState<TrackGeometrySvg | null>(null)
  const [sessionKey, setSessionKey] = useState(0)
  const prevLastLapMs = useRef<number | null | undefined>(undefined)
  const demoFramesRef = useRef<DemoFrame[]>([])
  const demoIndexRef = useRef(0)
  const demoStartRef = useRef<number | null>(null)
  const demoDurationRef = useRef(0)
  const demoElapsedRef = useRef(0)
  const demoTimerRef = useRef<number | null>(null)

  useEffect(() => {
    const currentLastLap = telemetry?.last_lap_ms
    const hadValidLap =
      prevLastLapMs.current !== undefined &&
      prevLastLapMs.current !== null &&
      prevLastLapMs.current > 0
    const isNowReset = currentLastLap === undefined || currentLastLap === null || currentLastLap === 0

    if (hadValidLap && isNowReset) {
      setSessionKey((k) => k + 1)
    }

    prevLastLapMs.current = currentLastLap
  }, [telemetry?.last_lap_ms])

  const prevInRace = useRef(false)
  const demoActiveRef = useRef(demoActive)
  useEffect(() => {
    const inRace = !!telemetry?.in_race
    if (inRace && !prevInRace.current) {
      setSessionKey((k) => k + 1)
    }
    prevInRace.current = inRace
  }, [telemetry?.in_race])
  useEffect(() => {
    demoActiveRef.current = demoActive
  }, [demoActive])

  const [rawTab, setRawTab] = useState<'encrypted' | 'decrypted'>('encrypted')
  const [debugView, setDebugView] = useState<'formatted' | 'raw'>('formatted')
  const carMetaCache = useRef<Map<number, MetaCarResponse>>(new Map())
  const trackMetaCache = useRef<Map<number, MetaTrackResponse>>(new Map())
  const trackGeometryCache = useRef<Map<number, TrackGeometrySvg>>(new Map())
  const [speedUnit, setSpeedUnit] = useState<SpeedUnit>(() =>
    getStoredUnit(UNIT_KEYS.speed, ['kph', 'mph'], 'kph'),
  )
  const [tempUnit, setTempUnit] = useState<TempUnit>(() =>
    getStoredUnit(UNIT_KEYS.temp, ['c', 'f'], 'c'),
  )
  const [pressureUnit, setPressureUnit] = useState<PressureUnit>(() =>
    getStoredUnit(UNIT_KEYS.pressure, ['kpa', 'psi'], 'kpa'),
  )
  const [fuelUnit, setFuelUnit] = useState<FuelUnit>(() =>
    getStoredUnit(UNIT_KEYS.fuel, ['l', 'gal'], 'l'),
  )
  const isPagesDemo =
    import.meta.env.VITE_PAGES_DEMO === 'true' ||
    (typeof window !== 'undefined' && window.location.hostname.endsWith('github.io'))
  const demoBinUrl = `${import.meta.env.BASE_URL}demo/demo_race.bin`
  const demoMetaUrl = `${import.meta.env.BASE_URL}demo/demo_race.meta.json`
  const wsUrl = import.meta.env.VITE_WS_URL || 'ws://127.0.0.1:10086/ws'
  const stopDemoPlayback = useCallback(() => {
    if (demoTimerRef.current !== null && typeof window !== 'undefined') {
      window.cancelAnimationFrame(demoTimerRef.current)
      demoTimerRef.current = null
    }
  }, [])

  const startDemoPlayback = useCallback(
    (frames: DemoFrame[]) => {
      if (typeof window === 'undefined' || frames.length === 0) {
        return
      }
      stopDemoPlayback()
      demoFramesRef.current = frames
      demoIndexRef.current = 0
      demoElapsedRef.current = 0
      demoStartRef.current = window.performance.now()
      demoDurationRef.current = frames[frames.length - 1]?.t_ms ?? 0

      const tick = () => {
        if (!demoActiveRef.current) {
          stopDemoPlayback()
          return
        }
        const duration = demoDurationRef.current || 1
        const now = window.performance.now()
        const start = demoStartRef.current ?? now
        const elapsed = (now - start) % duration

        if (elapsed < demoElapsedRef.current) {
          demoIndexRef.current = 0
        }
        demoElapsedRef.current = elapsed

        const framesRef = demoFramesRef.current
        let idx = demoIndexRef.current
        while (idx + 1 < framesRef.length && framesRef[idx + 1].t_ms <= elapsed) {
          idx += 1
        }
        demoIndexRef.current = idx

        const frame = framesRef[idx]
        if (frame) {
          setTelemetry(frame.state)
          const nowTs = Date.now()
          setLastTelemetryAt(nowTs)
          setDebugData(buildDebugFromState(frame.state, nowTs))
        }
        demoTimerRef.current = window.requestAnimationFrame(tick)
      }

      demoTimerRef.current = window.requestAnimationFrame(tick)
    },
    [stopDemoPlayback],
  )
  const pushLog = useCallback((level: UiLog['level'], message: string) => {
    setUiLogs((prev) => {
      const next = [...prev, { at: Date.now(), level, message }]
      if (next.length > 200) {
        next.shift()
      }
      return next
    })
  }, [])

  const nowMs = tickNow || Date.now()
  const telemetryActive = lastTelemetryAt !== null && nowMs - lastTelemetryAt < 2000

  const selectedLang = normalizeLang(i18n.language)
  const numberFormats = useMemo(() => createNumberFormats(selectedLang), [selectedLang])

  const tabs = [
    { key: 'race', label: t('tabs.race') },
    { key: 'tires', label: t('tabs.tires') },
    { key: 'dynamics', label: t('tabs.dynamics') },
    { key: 'debug', label: t('tabs.debug') },
    { key: 'settings', label: t('tabs.settings') },
  ] satisfies Array<{ key: TabKey; label: string }>

  useEffect(() => {
    if (isPagesDemo) {
      setStatus('demo')
      return
    }
    const socket = new WebSocket(wsUrl)
    setStatus('connecting')

    socket.onopen = () => {
      setStatus('open')
      pushLog('info', `ws open ${wsUrl}`)
    }
    socket.onclose = () => {
      setStatus('closed')
      pushLog('warn', 'ws closed')
    }
    socket.onerror = () => {
      setStatus('error')
      pushLog('error', 'ws error')
    }
    socket.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as TelemetryMessage
        if (!message || typeof message.type !== 'string') {
          return
        }
        if (message.type === 'state_update') {
          setTelemetry((prev) => {
            const {
              in_race,
              is_paused,
              packet_id,
              current_position,
              total_positions,
              current_lap,
              total_laps,
              best_lap_ms,
              last_lap_ms,
              time_on_track_ms,
              car_id,
              track_id,
              ...rest
            } = message.state
            const next = { ...prev }
            if (in_race !== undefined) {
              next.in_race = in_race
            }
            if (is_paused !== undefined) {
              next.is_paused = is_paused
            }
            if (packet_id !== undefined) {
              next.packet_id = packet_id
            }
            if (current_position !== undefined) {
              next.current_position = current_position
            }
            if (total_positions !== undefined) {
              next.total_positions = total_positions
            }
            if (current_lap !== undefined) {
              next.current_lap = current_lap
            }
            if (total_laps !== undefined) {
              next.total_laps = total_laps
            }
            if (best_lap_ms !== undefined) {
              next.best_lap_ms = best_lap_ms
            }
            if (last_lap_ms !== undefined) {
              next.last_lap_ms = last_lap_ms
            }
            if (time_on_track_ms !== undefined) {
              next.time_on_track_ms = time_on_track_ms
            }
            if (car_id !== undefined) {
              next.car_id = car_id
            }
            if (track_id !== undefined) {
              next.track_id = track_id
            }
            const canUpdate = (in_race ?? prev.in_race) === true || demoActiveRef.current
            if (canUpdate) {
              Object.assign(next, rest)
            }
            return next
          })
          setLastTelemetryAt(Date.now())
        }
        if (message.type === 'samples_window') {
          samplesWindowRef.current = {
            samples: message.window.samples,
            range: {
              start_ms: message.window.start_ms,
              end_ms: message.window.end_ms,
            },
            info: {
              count: message.window.samples.length,
              seconds: (message.window.end_ms - message.window.start_ms) / 1000,
            },
          }
        }
      } catch {
        return
      }
    }

    return () => {
      socket.close()
    }
  }, [wsUrl, pushLog, isPagesDemo])

  useEffect(() => {
    if (!isPagesDemo) {
      return
    }
    let cancelled = false
    const load = async () => {
      setDemoPending(true)
      setDemoError(null)
      try {
        const [metaRes, binRes] = await Promise.all([
          fetch(demoMetaUrl).catch(() => null),
          fetch(demoBinUrl),
        ])
        if (!binRes.ok) {
          throw new Error(`demo bin ${binRes.status}`)
        }
        const meta = metaRes && metaRes.ok ? ((await metaRes.json()) as DemoMeta) : undefined
        const binBuffer = await binRes.arrayBuffer()
        if (cancelled) {
          return
        }
        const frames = await decodeDemoBin(binBuffer, meta?.track?.id ?? null, meta?.car?.id ?? null)
        if (cancelled) {
          return
        }

        demoFramesRef.current = frames
        demoDurationRef.current = frames.length ? frames[frames.length - 1].t_ms : 0
        demoIndexRef.current = 0
        demoElapsedRef.current = 0
        demoStartRef.current = typeof window !== 'undefined' ? window.performance.now() : null

        if (meta?.car) {
          carMetaCache.current.set(meta.car.id, meta.car)
          setMetaCar(meta.car)
        }
        if (meta?.track) {
          trackMetaCache.current.set(meta.track.id, meta.track)
          setMetaTrack(meta.track)
        }
        if (meta?.geometry) {
          trackGeometryCache.current.set(meta.geometry.id, meta.geometry)
          setTrackGeometry(meta.geometry)
        }
        if (meta?.debug) {
          setDebugData(meta.debug)
        } else {
          setDebugData(emptyDebugData(Date.now()))
        }

        setDemoActive(true)
        setDemoPath(demoBinUrl)
        pushLog('info', `demo bin loaded frames=${frames.length}`)
        startDemoPlayback(frames)
      } catch (err) {
        if (!cancelled) {
          setDemoError(t('demo.errorStart'))
          pushLog('error', 'demo bin load failed')
        }
      } finally {
        if (!cancelled) {
          setDemoPending(false)
        }
      }
    }
    load()
    return () => {
      cancelled = true
      stopDemoPlayback()
    }
  }, [
    demoBinUrl,
    demoMetaUrl,
    decodeDemoBin,
    isPagesDemo,
    pushLog,
    startDemoPlayback,
    stopDemoPlayback,
    t,
  ])

  const fetchConfig = async () => {
    if (isPagesDemo) {
      return null
    }
    try {
      const res = await fetch('/config/udp')
      const data = (await res.json()) as UdpConfig
      setUdpConfig(data)
      setPs5Input(data.ps5_ip ?? '')
      setDetectIp(data.ps5_ip ?? null)
      return data
    } catch {
      setUdpConfig(null)
      return null
    }
  }

  const startAutoDetect = async () => {
    if (isPagesDemo) {
      return
    }
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
  }

  const fetchDemoStatus = async () => {
    if (isPagesDemo) {
      return
    }
    try {
      const res = await fetch('/demo/status')
      if (!res.ok) {
        throw new Error('demo status failed')
      }
      const data = (await res.json()) as DemoStatusResponse
      setDemoActive(data.active)
      setDemoPath(data.path ?? null)
      setDemoError(null)
      pushLog('info', `demo status active=${data.active} path=${data.path ?? '-'}`)
    } catch {
      setDemoError(t('demo.errorStatus'))
      pushLog('error', 'demo status failed')
    }
  }

  const toggleDemo = async () => {
    if (isPagesDemo) {
      if (demoActive) {
        setDemoActive(false)
        stopDemoPlayback()
      } else {
        setDemoActive(true)
        if (demoFramesRef.current.length > 0) {
          demoStartRef.current = typeof window !== 'undefined' ? window.performance.now() : null
          demoElapsedRef.current = 0
          demoIndexRef.current = 0
          startDemoPlayback(demoFramesRef.current)
        }
      }
      return
    }
    if (demoPending) {
      return
    }
    setDemoPending(true)
    setDemoError(null)
    const endpoint = demoActive ? '/demo/stop' : '/demo/start'
    try {
      pushLog('info', `demo request ${endpoint}`)
      const res = await fetch(endpoint, { method: 'POST' })
      if (!res.ok) {
        pushLog('error', `demo request failed status=${res.status}`)
        throw new Error('demo toggle failed')
      }
      const data = (await res.json()) as DemoStatusResponse
      setDemoActive(data.active)
      setDemoPath(data.path ?? null)
      pushLog('info', `demo response active=${data.active} path=${data.path ?? '-'}`)
    } catch {
      setDemoError(demoActive ? t('demo.errorStop') : t('demo.errorStart'))
    } finally {
      setDemoPending(false)
    }
  }

  useEffect(() => {
    const load = async () => {
      if (isPagesDemo) {
        return
      }
      await fetchConfig()
      await startAutoDetect()
    }
    load()
  }, [isPagesDemo])

  useEffect(() => {
    if (!isPagesDemo) {
      fetchDemoStatus()
    }
  }, [isPagesDemo])

  useEffect(() => {
    if (activeTab === 'settings' && !isPagesDemo) {
      fetchConfig()
      fetchDemoStatus()
    }
  }, [activeTab, isPagesDemo])

  const trackId = telemetry.track_id
  useEffect(() => {
    if (activeTab !== 'race') {
      return
    }
    if (isPagesDemo) {
      if (trackId === undefined) {
        setTrackGeometry(null)
        return
      }
      const cached = trackGeometryCache.current.get(trackId)
      if (cached) {
        setTrackGeometry(cached)
      }
      return
    }
    if (trackId === undefined) {
      setTrackGeometry(null)
      return
    }
    let cancelled = false
    const load = async () => {
      const tryFetch = async (id: number) => {
        const cached = trackGeometryCache.current.get(id)
        if (cached) {
          return cached
        }
        const res = await fetch(`/meta/track/${id}/geometry/svg`)
        if (!res.ok) {
          return null
        }
        const data = (await res.json()) as TrackGeometrySvg
        trackGeometryCache.current.set(id, data)
        return data
      }

      try {
        const primary = await tryFetch(trackId)
        if (cancelled) {
          return
        }
        if (primary?.exists) {
          setTrackGeometry(primary)
          return
        }
        const baseId = metaTrack?.base_id
        if (baseId !== undefined && baseId !== null && baseId !== trackId) {
          const fallback = await tryFetch(baseId)
          if (!cancelled) {
            setTrackGeometry(fallback ?? primary ?? null)
          }
          return
        }
        setTrackGeometry(primary ?? null)
      } catch {
        if (!cancelled) {
          setTrackGeometry(null)
        }
      }
    }
    load()
    return () => {
      cancelled = true
    }
  }, [activeTab, trackId, metaTrack?.base_id, isPagesDemo])

  useEffect(() => {
    if (activeTab !== 'debug' || isPagesDemo) {
      return
    }
    let cancelled = false
    const poll = async () => {
      try {
        const res = await fetch('/debug/telemetry')
        if (!res.ok) {
          return
        }
        const data = (await res.json()) as DebugTelemetryResponse
        if (!cancelled) {
          setDebugData(data)
        }
      } catch {
        if (!cancelled) {
          setDebugData(null)
        }
      }
    }
    poll()
    const timer = window.setInterval(poll, 1000)
    return () => {
      cancelled = true
      window.clearInterval(timer)
    }
  }, [activeTab, isPagesDemo])

  useEffect(() => {
    if (detectId === null) {
      return
    }
    if (isPagesDemo) {
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
          await fetchConfig()
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
  }, [detectId, isPagesDemo])

  useEffect(() => {
    const timer = window.setInterval(() => {
      setTickNow(Date.now())
    }, 1000)
    return () => window.clearInterval(timer)
  }, [])

  const carId = telemetry.car_id
  useEffect(() => {
    if (carId === undefined) {
      setMetaCar(null)
      return
    }
    const cached = carMetaCache.current.get(carId)
    if (cached) {
      setMetaCar(cached)
      return
    }
    if (isPagesDemo) {
      return
    }
    let cancelled = false
    const load = async () => {
      try {
        const res = await fetch(`/meta/car/${carId}`)
        if (!res.ok) {
          return
        }
        const data = (await res.json()) as MetaCarResponse
        if (!cancelled) {
          carMetaCache.current.set(carId, data)
          setMetaCar(data)
        }
      } catch {
        if (!cancelled) {
          setMetaCar(null)
        }
      }
    }
    load()
    return () => {
      cancelled = true
    }
  }, [carId, isPagesDemo])

  useEffect(() => {
    if (trackId === undefined) {
      setMetaTrack(null)
      return
    }
    const cached = trackMetaCache.current.get(trackId)
    if (cached) {
      setMetaTrack(cached)
      return
    }
    if (isPagesDemo) {
      return
    }
    let cancelled = false
    const load = async () => {
      try {
        const res = await fetch(`/meta/track/${trackId}`)
        if (!res.ok) {
          return
        }
        const data = (await res.json()) as MetaTrackResponse
        if (!cancelled) {
          trackMetaCache.current.set(trackId, data)
          setMetaTrack(data)
        }
      } catch {
        if (!cancelled) {
          setMetaTrack(null)
        }
      }
    }
    load()
    return () => {
      cancelled = true
    }
  }, [trackId, isPagesDemo])

  const applyLanguage = (value: string) => {
    const normalized = normalizeLang(value)
    i18n.changeLanguage(normalized)
    if (typeof window !== 'undefined') {
      window.localStorage.setItem('apextelemetry.lang', normalized)
    }
  }

  const updateUnit = <T extends string>(
    key: string,
    value: string,
    setter: (next: T) => void,
  ) => {
    const next = value as T
    setter(next)
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(key, next)
    }
  }

  const copyDebugJson = async () => {
    if (!debugData || !navigator.clipboard) {
      return
    }
    try {
      await navigator.clipboard.writeText(JSON.stringify(debugData, null, 2))
      setDebugCopied(true)
      window.setTimeout(() => setDebugCopied(false), 1500)
    } catch {
      setDebugCopied(false)
    }
  }

  const copyDebugRaw = async (text: string) => {
    if (!navigator.clipboard) {
      return
    }
    try {
      await navigator.clipboard.writeText(text)
      setDebugRawCopied(true)
      window.setTimeout(() => setDebugRawCopied(false), 1500)
    } catch {
      setDebugRawCopied(false)
    }
  }

  const applyManualBind = async () => {
    if (isPagesDemo) {
      return
    }
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
  }

  return (
    <>
      <div className="app" data-ws-status={status}>
        <header className="header">
          <div>
            <h1>{t('app.title')}</h1>
            <p className="subtitle">{t('app.subtitle')}</p>
          </div>
        </header>

        <nav className="tabs">
          {tabs.map((tab) => (
            <button
              key={tab.key}
              type="button"
              className={`tab tab-${tab.key} ${activeTab === tab.key ? 'active' : ''}`}
              onClick={() => setActiveTab(tab.key)}
            >
              {tab.label}
            </button>
          ))}
        </nav>

        {activeTab === 'race' && (
          <RaceTab
            telemetry={telemetry}
            metaCar={metaCar}
            metaTrack={metaTrack}
            trackGeometry={trackGeometry}
            demoActive={demoActive}
            telemetryActive={telemetryActive}
            sessionKey={sessionKey}
            speedUnit={speedUnit}
            numberFormats={numberFormats}
          />
        )}

        {activeTab === 'tires' && <TiresTab />}

        {activeTab === 'dynamics' && <DynamicsTab />}

        {activeTab === 'settings' && (
          <SettingsTab
            telemetryActive={telemetryActive}
            detectStatus={detectStatus}
            detectIp={detectIp}
            ps5Input={ps5Input}
            setPs5Input={setPs5Input}
            onStartAutoDetect={startAutoDetect}
            onManualBind={applyManualBind}
            demoActive={demoActive}
            demoPath={demoPath}
            demoPending={demoPending}
            demoError={demoError}
            onToggleDemo={toggleDemo}
            selectedLang={selectedLang}
            onApplyLanguage={applyLanguage}
            speedUnit={speedUnit}
            tempUnit={tempUnit}
            pressureUnit={pressureUnit}
            fuelUnit={fuelUnit}
            onUpdateUnit={updateUnit}
            setSpeedUnit={setSpeedUnit}
            setTempUnit={setTempUnit}
            setPressureUnit={setPressureUnit}
            setFuelUnit={setFuelUnit}
            uiDebugEnabled={uiDebugEnabled}
            setUiDebugEnabled={setUiDebugEnabled}
            uiLogs={uiLogs}
            onClearUiLogs={() => setUiLogs([])}
          />
        )}

        {activeTab === 'debug' && (
          <DebugTab
            debugData={debugData}
            debugView={debugView}
            rawTab={rawTab}
            debugCopied={debugCopied}
            debugRawCopied={debugRawCopied}
            speedUnit={speedUnit}
            tempUnit={tempUnit}
            pressureUnit={pressureUnit}
            fuelUnit={fuelUnit}
            numberFormats={numberFormats}
            selectedLang={selectedLang}
            onDebugViewChange={setDebugView}
            onRawTabChange={setRawTab}
            onCopyJson={copyDebugJson}
            onCopyRaw={copyDebugRaw}
          />
        )}
      </div>
    </>
  )
}

export default App
