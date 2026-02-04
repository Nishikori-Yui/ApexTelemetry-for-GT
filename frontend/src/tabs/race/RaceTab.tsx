import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { TrackMap } from '../../TrackMap'
import type {
  MetaCarResponse,
  MetaTrackResponse,
  SpeedUnit,
  TelemetryState,
  TrackGeometrySvg,
} from '../../types'
import type { NumberFormats } from '../../utils/format'
import { formatHMS, formatLap } from '../../utils/format'
import { toMph } from '../../utils/units'
import {
  buildFuelTicks,
  buildSpeedTicks,
  buildTachTicks,
  buildTurboMinorTicks,
  buildTurboTicks,
  describeArc,
  describeArcWithSweep,
  FUEL_GAUGE_CX,
  FUEL_GAUGE_CY,
  FUEL_GAUGE_PATH,
  FUEL_GAUGE_R,
  FUEL_GAUGE_SPAN,
  FUEL_GAUGE_START,
  SPEED_ARC_PATH,
  SPEED_ARC_SPAN,
  SPEED_ARC_START,
  TACH_ARC_PATH,
  TACH_ARC_SPAN,
  TACH_ARC_START,
  TACH_MAX_RPM,
  TACH_RED_START,
  TURBO_BASE_PATH,
  TURBO_CX,
  TURBO_CY_ACTUAL,
  TURBO_R,
  turboAngleForNorm,
} from './gauges'

const TURBO_MIN_NORM = -1
const TURBO_MAX_NORM = 2

const clamp01 = (value?: number) => {
  if (value === undefined || Number.isNaN(value)) {
    return 0
  }
  return Math.min(Math.max(value, 0), 1)
}

type RaceTabProps = {
  telemetry: TelemetryState
  metaCar: MetaCarResponse | null
  metaTrack: MetaTrackResponse | null
  trackGeometry: TrackGeometrySvg | null
  demoActive: boolean
  telemetryActive: boolean
  sessionKey: number
  speedUnit: SpeedUnit
  numberFormats: NumberFormats
}

export function RaceTab({
  telemetry,
  metaCar,
  metaTrack,
  trackGeometry,
  demoActive,
  telemetryActive,
  sessionKey,
  speedUnit,
  numberFormats,
}: RaceTabProps) {
  const { t } = useTranslation()

  const speedUnitLabel = speedUnit === 'mph' ? t('units.mph') : t('units.kph')
  const speedDialMax = speedUnit === 'mph' ? 200 : 320
  const speedThreshold = speedUnit === 'mph' ? 100 : 160
  const speedMajorStep = speedUnit === 'mph' ? 20 : 40
  const speedMinorStep = speedUnit === 'mph' ? 10 : 20
  const speedValue =
    telemetry.speed_kph !== undefined
      ? speedUnit === 'mph'
        ? toMph(telemetry.speed_kph)
        : telemetry.speed_kph
      : undefined
  const speedRatio =
    speedValue === undefined
      ? 0
      : Math.min(Math.max(speedValue / speedDialMax, 0), 1)
  const speedThresholdRatio = Math.min(
    Math.max(speedThreshold / speedDialMax, 0),
    1,
  )
  const speedThresholdAngle = SPEED_ARC_START + speedThresholdRatio * SPEED_ARC_SPAN
  const speedLowAngle = SPEED_ARC_START + Math.min(speedRatio, speedThresholdRatio) * SPEED_ARC_SPAN
  const speedHighAngle =
    speedRatio > speedThresholdRatio
      ? SPEED_ARC_START + speedRatio * SPEED_ARC_SPAN
      : speedThresholdAngle
  const speedTickData = useMemo(
    () => buildSpeedTicks(speedDialMax, speedMajorStep, speedMinorStep),
    [speedDialMax, speedMajorStep, speedMinorStep],
  )

  const rpmValue = telemetry.rpm
  const throttleRatio = clamp01(telemetry.throttle)
  const brakeRatio = clamp01(telemetry.brake)
  const boostKpa = telemetry.boost_kpa

  const boostNorm =
    boostKpa === undefined
      ? undefined
      : Math.max(TURBO_MIN_NORM, Math.min(boostKpa / 100, TURBO_MAX_NORM))

  const turboAngle = boostNorm === undefined ? null : turboAngleForNorm(boostNorm)

  const turboNegPath =
    boostNorm !== undefined && boostNorm < 0
      ? describeArc(TURBO_CX, TURBO_CY_ACTUAL, TURBO_R, turboAngle!, 270)
      : null

  const turboPosPath =
    boostNorm !== undefined && boostNorm > 0
      ? describeArc(TURBO_CX, TURBO_CY_ACTUAL, TURBO_R, 270, turboAngle!)
      : null

  const turboTicks = useMemo(() => buildTurboTicks(), [])
  const turboMinorTicks = useMemo(() => buildTurboMinorTicks(), [])
  const fuelTicks = useMemo(() => buildFuelTicks(), [])
  const tachTickData = useMemo(() => buildTachTicks(), [])

  const tachRatio =
    rpmValue === undefined
      ? 0
      : Math.min(Math.max(rpmValue / TACH_MAX_RPM, 0), 1)
  const tachRedRatio = TACH_RED_START / TACH_MAX_RPM
  const tachRedStartAngle = TACH_ARC_START + tachRedRatio * TACH_ARC_SPAN
  const tachAngle = TACH_ARC_START + tachRatio * TACH_ARC_SPAN
  const tachNormalAngle = Math.min(tachAngle, tachRedStartAngle)
  const tachRedAngle = tachAngle > tachRedStartAngle ? tachAngle : tachRedStartAngle

  const fuelLiters = telemetry.fuel_l
  const fuelCapacity = telemetry.fuel_capacity_l
  const fuelPct =
    fuelLiters !== undefined && fuelCapacity !== undefined && fuelCapacity > 0
      ? Math.min(Math.max((fuelLiters / fuelCapacity) * 100, 0), 100)
      : undefined
  const fuelGaugeRatio = fuelPct !== undefined ? fuelPct / 100 : 0
  const fuelGaugeEnd = FUEL_GAUGE_START + fuelGaugeRatio * FUEL_GAUGE_SPAN
  const fuelGaugePath = describeArcWithSweep(
    FUEL_GAUGE_CX,
    FUEL_GAUGE_CY,
    FUEL_GAUGE_R,
    FUEL_GAUGE_START,
    fuelGaugeEnd,
    1,
  )

  const raceActive = telemetry.in_race === true || (demoActive && telemetryActive)
  const isPaused = telemetry.is_paused === true
  const modeLabel = raceActive
    ? isPaused
      ? t('status.paused')
      : t('status.inRace')
    : t('status.notInRace')
  const modeClass = raceActive
    ? isPaused
      ? 'paused'
      : 'in-race'
    : 'not-in-race'
  const timeOnTrack = formatHMS(telemetry.time_on_track_ms)
  const carId = telemetry.car_id
  const trackId = telemetry.track_id
  const carLabel =
    carId !== undefined
      ? `${numberFormats.int.format(carId)}${metaCar?.name ? ` · ${metaCar.name}` : ''}`
      : '—'
  const trackLabel =
    trackId !== undefined
      ? `${numberFormats.int.format(trackId)}${metaTrack?.name ? ` · ${metaTrack.name}` : ''}`
      : '—'
  const lapLabel =
    telemetry.current_lap !== undefined && telemetry.total_laps !== undefined
      ? `${numberFormats.int.format(
          Math.min(telemetry.current_lap, telemetry.total_laps),
        )} / ${numberFormats.int.format(telemetry.total_laps)}`
      : '—'
  const positionLabel =
    telemetry.current_position !== undefined && telemetry.total_positions !== undefined
      ? `${numberFormats.int.format(
          telemetry.current_position,
        )} / ${numberFormats.int.format(telemetry.total_positions)}`
      : '—'
  const bestLapLabel = formatLap(telemetry.best_lap_ms)
  const lastLapLabel = formatLap(telemetry.last_lap_ms)

  const fuelPctDisplay =
    fuelPct === undefined ? '—' : numberFormats.fuelPct.format(fuelPct)
  const rpmDisplay =
    rpmValue === undefined ? '—' : numberFormats.rpm.format(rpmValue)
  const speedDisplay =
    speedValue === undefined
      ? '—'
      : numberFormats.int.format(Math.round(speedValue))
  const gearDisplay =
    telemetry.gear === undefined
      ? '—'
      : telemetry.gear === 0
        ? 'R'
        : telemetry.gear.toString()
  const boostDisplay = boostKpa !== undefined ? (boostKpa / 100).toFixed(2) : '—'
  const avgConsumePerLap = telemetry.avg_fuel_consume_pct_per_lap
  const lapsLeft = telemetry.fuel_laps_remaining
  const lapsLeftDisplay =
    lapsLeft === undefined ? '—' : numberFormats.fuelPct.format(lapsLeft)
  const consumeDisplay =
    avgConsumePerLap === undefined
      ? '—'
      : numberFormats.fuelPct.format(avgConsumePerLap)
  const lowFuel = lapsLeft !== undefined && lapsLeft < 1

  return (
    <>
      <section className="panel info-panel">
        <div className="info-item">
          <span className="info-label">{t('status.car')}</span>
          <span className="info-value">
            {carLabel.includes('·')
              ? carLabel.split('·')[1].trim()
              : carLabel.includes(':')
                ? carLabel.split(':')[1].trim()
                : carLabel}
          </span>
        </div>
        <div className="info-item">
          <span className="info-label">{t('status.track')}</span>
          <span className="info-value">{trackLabel}</span>
        </div>
      </section>

      <section className="panel race-status-panel">
        <div className="race-status-top">
          <div className="race-status-center">
            <span className={`race-mode ${modeClass}`}>{modeLabel}</span>
            {demoActive && <span className="demo-badge">{t('status.demo')}</span>}
          </div>
        </div>
        <div className="race-status-grid">
          <div className="race-stat">
            <span className="race-stat-label">{t('status.trackTime')}</span>
            <span className="race-stat-value fixed-hms">{timeOnTrack}</span>
          </div>
          <div className="race-stat">
            <span className="race-stat-label">{t('status.position')}</span>
            <span className="race-stat-value fixed-position">{positionLabel}</span>
          </div>
          <div className="race-stat">
            <span className="race-stat-label">{t('status.lap')}</span>
            <span className="race-stat-value fixed-lap">{lapLabel}</span>
          </div>
          <div className="race-stat">
            <span className="race-stat-label">{t('status.bestLap')}</span>
            <span className="race-stat-value fixed-lap-time">{bestLapLabel}</span>
          </div>
          <div className="race-stat">
            <span className="race-stat-label">{t('status.lastLap')}</span>
            <span className="race-stat-value fixed-lap-time">{lastLapLabel}</span>
          </div>
        </div>
      </section>

      <section className="panel cluster-panel">
        {!raceActive && (
          <div className="cluster-overlay">
            <div className="overlay-content">Waiting for Race...</div>
          </div>
        )}

        <div className="cluster-grid">
          <div className="cluster-gauge speed-gauge">
            <div className="gauge-shell">
              <div className="gauge-dial speed-dial">
                <svg className="speed-ticks" viewBox="0 0 220 220" aria-hidden="true">
                  {speedTickData.ticks.map((tick) => (
                    <line
                      key={`${tick.value}-${tick.kind}`}
                      className={`speed-tick ${tick.kind}`}
                      x1={tick.x1}
                      y1={tick.y1}
                      x2={tick.x2}
                      y2={tick.y2}
                    />
                  ))}
                  {speedTickData.labels.map((label) => (
                    <text
                      key={`label-${label.value}`}
                      className="speed-label"
                      x={label.x}
                      y={label.y}
                      textAnchor="middle"
                      dominantBaseline="middle"
                    >
                      {label.value}
                    </text>
                  ))}
                  <path className="speed-arc-bg" fill="none" d={SPEED_ARC_PATH} />
                  <path
                    className="speed-arc-zone"
                    fill="none"
                    d={describeArc(110, 110, 90, SPEED_ARC_START, speedThresholdAngle)}
                  />
                  <path
                    className="speed-arc-zone speed-arc-zone-high"
                    d={describeArc(
                      110,
                      110,
                      90,
                      speedThresholdAngle,
                      SPEED_ARC_START + SPEED_ARC_SPAN,
                    )}
                  />
                  <path
                    className="speed-arc-progress"
                    fill="none"
                    d={describeArc(110, 110, 90, SPEED_ARC_START, speedLowAngle)}
                  />
                  {speedRatio > speedThresholdRatio && (
                    <path
                      className="speed-arc-progress speed-arc-progress-high"
                      d={describeArc(110, 110, 90, speedThresholdAngle, speedHighAngle)}
                    />
                  )}
                </svg>
                <div className="speed-center">
                  <div className="speed-value cluster-numeric">{speedDisplay}</div>
                  <div className="speed-unit">{speedUnitLabel}</div>
                </div>
              </div>
            </div>
          </div>
          <div className="cluster-bar brake-bar">
            <div className="bar-shell">
              <div className="bar-track">
                <div
                  className="bar-fill brake-fill"
                  style={{ height: `${Math.round(brakeRatio * 100)}%` }}
                />
                <div className="bar-midline" />
              </div>
              <div className="bar-label">{t('gauges.brake')}</div>
            </div>
          </div>
          <div className="cluster-center">
            <div className="center-shell">
              <div className="center-top-slot">
                <span className="last-lap-time">{lastLapLabel}</span>
              </div>
              <div className="center-mid-slot">{gearDisplay}</div>
              <div className="center-bot-slot">
                <span className="current-lap-time">
                  {formatLap(telemetry.current_lap_time_ms)}
                </span>
              </div>
            </div>
          </div>
          <div className="cluster-bar throttle-bar">
            <div className="bar-shell">
              <div className="bar-track">
                <div
                  className="bar-fill throttle-fill"
                  style={{ height: `${Math.round(throttleRatio * 100)}%` }}
                />
                <div className="bar-midline" />
              </div>
              <div className="bar-label">{t('gauges.throttle')}</div>
            </div>
          </div>
          <div className="cluster-gauge tach-gauge-group">
            <div className="tach-gauge">
              <div className="gauge-shell">
                <div className="gauge-dial tach-dial">
                  <svg className="tach-arc" viewBox="0 0 220 220" aria-hidden="true">
                    <path className="tach-arc-base" fill="none" d={TACH_ARC_PATH} />
                    {tachTickData.ticks.map((tick, index) => (
                      <line
                        key={`tach-tick-${index}`}
                        className={`tach-tick ${tick.kind}`}
                        x1={tick.x1}
                        y1={tick.y1}
                        x2={tick.x2}
                        y2={tick.y2}
                      />
                    ))}
                    {tachTickData.labels.map((label) => (
                      <text
                        key={`tach-label-${label.value}`}
                        className="tach-label"
                        x={label.x}
                        y={label.y}
                        textAnchor="middle"
                        dominantBaseline="middle"
                      >
                        {label.value}
                      </text>
                    ))}
                    <path
                      className="tach-arc-redzone"
                      fill="none"
                      d={describeArc(110, 110, 90, tachRedStartAngle, TACH_ARC_START + TACH_ARC_SPAN)}
                    />
                    <path
                      className="tach-arc-progress"
                      fill="none"
                      d={describeArc(110, 110, 90, TACH_ARC_START, tachNormalAngle)}
                    />
                    {tachAngle > tachRedStartAngle && (
                      <path
                        className="tach-arc-progress tach-arc-progress-red"
                        d={describeArc(110, 110, 90, tachRedStartAngle, tachRedAngle)}
                      />
                    )}
                  </svg>
                  <div className="tach-center">
                    <div className="tach-value cluster-numeric">{rpmDisplay}</div>
                    <div className="tach-unit">{t('units.rpm')}</div>
                  </div>
                  <div className="boost-sub-gauge-slot hidden-slot" />
                </div>
              </div>
            </div>
            <div className="turbo-gauge-cluster">
              <div className="turbo-gauge-body">
                <div className="gauge-dial">
                  <svg className="turbo-gauge-svg" viewBox="0 0 260 200" aria-hidden="true">
                    <path className="turbo-arc-base" fill="none" d={TURBO_BASE_PATH} />
                    {turboNegPath && <path className="turbo-arc-neg" fill="none" d={turboNegPath} />}
                    {turboPosPath && <path className="turbo-arc-pos" fill="none" d={turboPosPath} />}
                    {turboTicks.map((tick) => (
                      <g key={tick.label}>
                        <line
                          className={`turbo-tick ${tick.label === '0' ? 'zero' : ''}`}
                          x1={tick.x1}
                          y1={tick.y1}
                          x2={tick.x2}
                          y2={tick.y2}
                        />
                        <text
                          className="turbo-tick-label"
                          x={tick.tx}
                          y={tick.ty}
                          textAnchor="middle"
                          dominantBaseline="middle"
                        >
                          {tick.label}
                        </text>
                      </g>
                    ))}
                    {turboMinorTicks.map((tick, i) => (
                      <line
                        key={`minor-${i}`}
                        className="turbo-tick minor"
                        x1={tick.x1}
                        y1={tick.y1}
                        x2={tick.x2}
                        y2={tick.y2}
                      />
                    ))}
                    <text
                      className="turbo-scale-label"
                      x="180"
                      y="80"
                      textAnchor="middle"
                      fill="var(--text-muted)"
                      style={{ fontSize: '12px' }}
                    >
                      &times;100 kPa
                    </text>
                  </svg>
                  <div className="turbo-value cluster-numeric">{boostDisplay}</div>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div className="fuel-row">
          <div className={`fuel-panel fuel-gauge-panel${lowFuel ? ' low-fuel' : ''}`}>
            <div className="fuel-panel-header">
              <h3>{t('fuelPanel.title')}</h3>
            </div>
            <div className={`fuel-gauge-body${fuelPct === undefined ? ' fuel-unknown' : ''}`}>
              <div className="gauge-dial">
                <svg className="fuel-gauge-svg" viewBox="0 0 260 180" aria-hidden="true">
                  <path className="fuel-gauge-base" fill="none" d={FUEL_GAUGE_PATH} />
                  <path className="fuel-gauge-fill" fill="none" d={fuelGaugePath} />
                  {fuelTicks.map((tick, i) => (
                    <line
                      key={i}
                      className={`fuel-tick ${tick.major ? 'major' : 'minor'}`}
                      x1={tick.x1}
                      y1={tick.y1}
                      x2={tick.x2}
                      y2={tick.y2}
                    />
                  ))}
                  <text className="fuel-gauge-scale" x="60" y="124">
                    E
                  </text>
                  <text className="fuel-gauge-scale" x="200" y="124">
                    F
                  </text>
                  <g className="fuel-gauge-icon" transform="translate(118 112)">
                    <path d="M8 0h9c1.5 0 2.7 1.2 2.7 2.7v2h3.3v16h-4v3H5V3C5 1.4 6.4 0 8 0zm0 6v12h10V6H8zm12 2v8h1V8h-1z" />
                  </g>
                </svg>
                <div className="fuel-gauge-percent cluster-numeric">{fuelPctDisplay}</div>
              </div>
            </div>
            <div className="fuel-stats">
              <div className="fuel-stat">
                <span className="fuel-stat-label">{t('fuelPanel.consumePerLap')}</span>
                <span className="fuel-stat-value">{consumeDisplay}</span>
              </div>
              <div className="fuel-stat">
                <span className="fuel-stat-label">{t('fuelPanel.lapsLeft')}</span>
                <span className="fuel-stat-value">{lapsLeftDisplay}</span>
              </div>
            </div>
          </div>
          <div className="fuel-panel track-map-panel">
            <div className="track-map-header">
              <h3>{t('trackMap.title')}</h3>
            </div>
            <div className="track-map-body">
              <TrackMap
                key={sessionKey}
                className="track-map-svg"
                trackGeometry={
                  trackGeometry?.exists && trackGeometry.view_box
                    ? {
                        view_box: trackGeometry.view_box,
                        path_d: trackGeometry.path_d ?? '',
                      }
                    : undefined
                }
                currentPos={
                  telemetry.pos_x !== undefined && telemetry.pos_z !== undefined
                    ? {
                        x: telemetry.pos_x,
                        z: telemetry.pos_z,
                        rotation: (() => {
                          const vx = telemetry.vel_x
                          const vz = telemetry.vel_z
                          if (vx !== undefined && vz !== undefined) {
                            const speed = Math.sqrt(vx * vx + vz * vz)
                            if (speed > 1.0) {
                              return (Math.atan2(vx, -vz) * 180) / Math.PI
                            }
                          }
                          return telemetry.rotation_yaw
                            ? (telemetry.rotation_yaw * 180) / Math.PI
                            : undefined
                        })(),
                      }
                    : undefined
                }
              />
            </div>
          </div>
        </div>
      </section>
    </>
  )
}
