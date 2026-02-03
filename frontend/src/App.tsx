import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import './App.css'
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
  const wsUrl = import.meta.env.VITE_WS_URL || 'ws://127.0.0.1:10086/ws'
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
  }, [wsUrl, pushLog])

  const fetchConfig = async () => {
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
      await fetchConfig()
      await startAutoDetect()
    }
    load()
  }, [])

  useEffect(() => {
    fetchDemoStatus()
  }, [])

  useEffect(() => {
    if (activeTab === 'settings') {
      fetchConfig()
      fetchDemoStatus()
    }
  }, [activeTab])

  const trackId = telemetry.track_id
  useEffect(() => {
    if (activeTab !== 'race') {
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
  }, [activeTab, trackId, metaTrack?.base_id])

  useEffect(() => {
    if (activeTab !== 'debug') {
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
  }, [activeTab])

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
  }, [detectId])

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
  }, [carId])

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
  }, [trackId])

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
