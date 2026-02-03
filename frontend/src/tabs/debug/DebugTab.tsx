import { useTranslation } from 'react-i18next'
import type {
  DebugTelemetryResponse,
  FuelUnit,
  PressureUnit,
  SpeedUnit,
  TempUnit,
} from '../../types'
import type { NumberFormats } from '../../utils/format'
import { formatHMS, formatLap } from '../../utils/format'
import { toFahrenheit, toGallons, toMph, toPsi } from '../../utils/units'

type DebugTabProps = {
  debugData: DebugTelemetryResponse | null
  debugView: 'formatted' | 'raw'
  rawTab: 'encrypted' | 'decrypted'
  debugCopied: boolean
  debugRawCopied: boolean
  speedUnit: SpeedUnit
  tempUnit: TempUnit
  pressureUnit: PressureUnit
  fuelUnit: FuelUnit
  numberFormats: NumberFormats
  selectedLang: string
  onDebugViewChange: (next: 'formatted' | 'raw') => void
  onRawTabChange: (next: 'encrypted' | 'decrypted') => void
  onCopyJson: () => void
  onCopyRaw: (text: string) => void
}

export function DebugTab({
  debugData,
  debugView,
  rawTab,
  debugCopied,
  debugRawCopied,
  speedUnit,
  tempUnit,
  pressureUnit,
  fuelUnit,
  numberFormats,
  selectedLang,
  onDebugViewChange,
  onRawTabChange,
  onCopyJson,
  onCopyRaw,
}: DebugTabProps) {
  const { t } = useTranslation()

  const formatOptional = (
    value: number | null | undefined,
    formatter: Intl.NumberFormat,
    suffix?: string,
  ) => {
    if (value === null || value === undefined) {
      return '—'
    }
    return suffix ? `${formatter.format(value)} ${suffix}` : formatter.format(value)
  }

  const formatOptionalRaw = (value: number | null | undefined, suffix?: string) =>
    value === null || value === undefined
      ? '—'
      : formatOptional(value, numberFormats.raw, suffix)

  const formatOptionalRatio = (value: number | null | undefined) =>
    value === null || value === undefined ? '—' : numberFormats.ratio.format(value)

  const formatOptionalRpm = (value: number | null | undefined) =>
    formatOptional(value, numberFormats.rpm, t('units.rpm'))

  const formatOptionalPercent = (value: number | null | undefined) => {
    if (value === null || value === undefined) {
      return '—'
    }
    return `${numberFormats.percent.format(value * 100)}%`
  }

  const formatOptionalBool = (value: boolean | null | undefined) => {
    if (value === null || value === undefined) {
      return '—'
    }
    return value ? t('debug.true') : t('debug.false')
  }

  const formatOptionalSpeed = (value: number | null | undefined) => {
    if (value === null || value === undefined) {
      return '—'
    }
    const speed = speedUnit === 'mph' ? toMph(value) : value
    const speedUnitLabel = speedUnit === 'mph' ? t('units.mph') : t('units.kph')
    return formatOptional(speed, numberFormats.speed, speedUnitLabel)
  }

  const formatOptionalTemp = (value: number | null | undefined) => {
    if (value === null || value === undefined) {
      return '—'
    }
    const temp = tempUnit === 'f' ? toFahrenheit(value) : value
    const unit = tempUnit === 'f' ? t('units.f') : t('units.c')
    return formatOptional(temp, numberFormats.speed, unit)
  }

  const formatOptionalPressure = (value: number | null | undefined) => {
    if (value === null || value === undefined) {
      return '—'
    }
    const pressure = pressureUnit === 'psi' ? toPsi(value) : value
    const unit = pressureUnit === 'psi' ? t('units.psi') : t('units.kpa')
    return formatOptional(pressure, numberFormats.speed, unit)
  }

  const formatOptionalFuel = (value: number | null | undefined) => {
    if (value === null || value === undefined) {
      return '—'
    }
    const fuel = fuelUnit === 'gal' ? toGallons(value) : value
    const unit = fuelUnit === 'gal' ? t('units.gal') : t('units.l')
    return formatOptional(fuel, numberFormats.fuel, unit)
  }

  const formatOptionalInt = (value: number | null | undefined) =>
    value === null || value === undefined ? '—' : numberFormats.int.format(value)

  const formatOptionalHex = (value: number | null | undefined, width = 2) => {
    if (value === null || value === undefined) {
      return '—'
    }
    return `0x${value.toString(16).padStart(width, '0').toUpperCase()}`
  }

  const formatOptionalTimestamp = (value: number | null | undefined) => {
    if (value === null || value === undefined) {
      return '—'
    }
    try {
      return new Date(value).toLocaleString(selectedLang)
    } catch {
      return numberFormats.int.format(value)
    }
  }

  const formatLapCount = (
    current: number | null | undefined,
    total: number | null | undefined,
  ) => {
    if (current === null || current === undefined || total === null || total === undefined) {
      return '—'
    }
    const safeCurrent = total > 0 ? Math.min(current, total) : current
    return `${numberFormats.int.format(safeCurrent)} / ${numberFormats.int.format(total)}`
  }

  const debugSession = debugData?.session
  const debugPowertrain = debugData?.powertrain
  const debugFluids = debugData?.fluids
  const debugTyres = debugData?.tyres
  const debugWheels = debugData?.wheels
  const debugChassis = debugData?.chassis
  const debugGears = debugData?.gears
  const debugDynamics = debugData?.dynamics
  const debugFlags = debugData?.flags
  const debugRaw = debugData?.raw
  const rawText =
    rawTab === 'encrypted'
      ? debugRaw?.encrypted_hex ?? '—'
      : debugRaw?.decrypted_hex ?? '—'

  return (
    <section className="panel tab-panel debug-panel">
      <div className="panel-header">
        <h2>{t('tabs.debug')}</h2>
        <div className="debug-actions">
          <div className="debug-view-toggle">
            <button
              type="button"
              className={debugView === 'formatted' ? 'active' : ''}
              onClick={() => onDebugViewChange('formatted')}
            >
              {t('debug.viewFormatted')}
            </button>
            <button
              type="button"
              className={debugView === 'raw' ? 'active' : ''}
              onClick={() => onDebugViewChange('raw')}
            >
              {t('debug.viewRaw')}
            </button>
          </div>
          {debugView === 'formatted' ? (
            <button type="button" onClick={onCopyJson}>
              {debugCopied ? t('debug.copied') : t('debug.copy')}
            </button>
          ) : (
            <button type="button" onClick={() => onCopyRaw(rawText)}>
              {debugRawCopied ? t('debug.copied') : t('debug.copyRaw')}
            </button>
          )}
        </div>
      </div>
      {debugView === 'formatted' ? (
        <div className="debug-grid">
          <div className="debug-card">
            <h3>{t('debug.section.session')}</h3>
            <div className="debug-rows">
              <div className="debug-row">
                <span>{t('debug.inRace')}</span>
                <span className="debug-value">{formatOptionalBool(debugSession?.in_race)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.isPaused')}</span>
                <span className="debug-value">{formatOptionalBool(debugSession?.is_paused)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.packetId')}</span>
                <span className="debug-value">{formatOptionalInt(debugSession?.packet_id)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.vehicleId')}</span>
                <span className="debug-value">{formatOptionalInt(debugSession?.car_id)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.trackId')}</span>
                <span className="debug-value">{formatOptionalInt(debugSession?.track_id)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.timeOnTrack')}</span>
                <span className="debug-value">
                  {formatHMS(debugSession?.time_on_track_ms ?? undefined)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.currentLap')}</span>
                <span className="debug-value">
                  {formatLapCount(debugSession?.current_lap, debugSession?.total_laps)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.position')}</span>
                <span className="debug-value">
                  {formatLapCount(debugSession?.current_position, debugSession?.total_positions)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.bestLap')}</span>
                <span className="debug-value">
                  {formatLap(debugSession?.best_lap_ms ?? undefined)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.lastLap')}</span>
                <span className="debug-value">
                  {formatLap(debugSession?.last_lap_ms ?? undefined)}
                </span>
              </div>
            </div>
          </div>

          <div className="debug-card">
            <h3>{t('debug.section.powertrain')}</h3>
            <div className="debug-rows">
              <div className="debug-row">
                <span>{t('debug.speed')}</span>
                <span className="debug-value">{formatOptionalSpeed(debugPowertrain?.speed_kph)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.rpm')}</span>
                <span className="debug-value">{formatOptionalRpm(debugPowertrain?.rpm)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.rpmRevWarning')}</span>
                <span className="debug-value">
                  {formatOptionalRpm(debugPowertrain?.rpm_rev_warning)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.rpmRevLimiter')}</span>
                <span className="debug-value">
                  {formatOptionalRpm(debugPowertrain?.rpm_rev_limiter)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.gear')}</span>
                <span className="debug-value">{formatOptionalInt(debugPowertrain?.gear)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.gearRaw')}</span>
                <span className="debug-value">{formatOptionalInt(debugPowertrain?.gear_raw)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.suggestedGear')}</span>
                <span className="debug-value">
                  {formatOptionalInt(debugPowertrain?.suggested_gear)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.throttle')}</span>
                <span className="debug-value">
                  {formatOptionalPercent(debugPowertrain?.throttle)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.brake')}</span>
                <span className="debug-value">{formatOptionalPercent(debugPowertrain?.brake)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.clutch')}</span>
                <span className="debug-value">{formatOptionalPercent(debugPowertrain?.clutch)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.clutchEngaged')}</span>
                <span className="debug-value">
                  {formatOptionalPercent(debugPowertrain?.clutch_engaged)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.rpmAfterClutch')}</span>
                <span className="debug-value">
                  {formatOptionalRpm(debugPowertrain?.rpm_after_clutch)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.boost')}</span>
                <span className="debug-value">
                  {formatOptionalPressure(debugPowertrain?.boost_kpa)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.estimatedSpeed')}</span>
                <span className="debug-value">
                  {formatOptionalSpeed(debugPowertrain?.estimated_speed_kph)}
                </span>
              </div>
            </div>
          </div>

          <div className="debug-card">
            <h3>{t('debug.section.fluids')}</h3>
            <div className="debug-rows">
              <div className="debug-row">
                <span>{t('debug.fuel')}</span>
                <span className="debug-value">{formatOptionalFuel(debugFluids?.fuel_l)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.fuelCapacity')}</span>
                <span className="debug-value">
                  {formatOptionalFuel(debugFluids?.fuel_capacity_l)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.oilTemp')}</span>
                <span className="debug-value">{formatOptionalTemp(debugFluids?.oil_temp_c)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.waterTemp')}</span>
                <span className="debug-value">{formatOptionalTemp(debugFluids?.water_temp_c)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.oilPressure')}</span>
                <span className="debug-value">
                  {formatOptionalPressure(debugFluids?.oil_pressure_kpa)}
                </span>
              </div>
            </div>
          </div>

          <div className="debug-card">
            <h3>{t('debug.section.tyres')}</h3>
            <div className="debug-rows">
              <div className="debug-row">
                <span>{t('debug.tyreFl')}</span>
                <span className="debug-value">{formatOptionalTemp(debugTyres?.temp_fl_c)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.tyreFr')}</span>
                <span className="debug-value">{formatOptionalTemp(debugTyres?.temp_fr_c)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.tyreRl')}</span>
                <span className="debug-value">{formatOptionalTemp(debugTyres?.temp_rl_c)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.tyreRr')}</span>
                <span className="debug-value">{formatOptionalTemp(debugTyres?.temp_rr_c)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.tyreDiamFl')}</span>
                <span className="debug-value">
                  {formatOptionalRaw(debugTyres?.tyre_diameter_fl_m, t('units.m'))}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.tyreDiamFr')}</span>
                <span className="debug-value">
                  {formatOptionalRaw(debugTyres?.tyre_diameter_fr_m, t('units.m'))}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.tyreDiamRl')}</span>
                <span className="debug-value">
                  {formatOptionalRaw(debugTyres?.tyre_diameter_rl_m, t('units.m'))}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.tyreDiamRr')}</span>
                <span className="debug-value">
                  {formatOptionalRaw(debugTyres?.tyre_diameter_rr_m, t('units.m'))}
                </span>
              </div>
            </div>
          </div>

          <div className="debug-card">
            <h3>{t('debug.section.wheels')}</h3>
            <div className="debug-rows">
              <div className="debug-row">
                <span>{t('debug.wheelSpeedFl')}</span>
                <span className="debug-value">{formatOptionalRaw(debugWheels?.wheel_speed_fl)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.wheelSpeedFr')}</span>
                <span className="debug-value">{formatOptionalRaw(debugWheels?.wheel_speed_fr)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.wheelSpeedRl')}</span>
                <span className="debug-value">{formatOptionalRaw(debugWheels?.wheel_speed_rl)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.wheelSpeedRr')}</span>
                <span className="debug-value">{formatOptionalRaw(debugWheels?.wheel_speed_rr)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.tyreSpeedFl')}</span>
                <span className="debug-value">
                  {formatOptionalSpeed(debugWheels?.tyre_speed_fl_kph)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.tyreSpeedFr')}</span>
                <span className="debug-value">
                  {formatOptionalSpeed(debugWheels?.tyre_speed_fr_kph)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.tyreSpeedRl')}</span>
                <span className="debug-value">
                  {formatOptionalSpeed(debugWheels?.tyre_speed_rl_kph)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.tyreSpeedRr')}</span>
                <span className="debug-value">
                  {formatOptionalSpeed(debugWheels?.tyre_speed_rr_kph)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.tyreSlipRatioFl')}</span>
                <span className="debug-value">
                  {formatOptionalRatio(debugWheels?.tyre_slip_ratio_fl)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.tyreSlipRatioFr')}</span>
                <span className="debug-value">
                  {formatOptionalRatio(debugWheels?.tyre_slip_ratio_fr)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.tyreSlipRatioRl')}</span>
                <span className="debug-value">
                  {formatOptionalRatio(debugWheels?.tyre_slip_ratio_rl)}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.tyreSlipRatioRr')}</span>
                <span className="debug-value">
                  {formatOptionalRatio(debugWheels?.tyre_slip_ratio_rr)}
                </span>
              </div>
            </div>
          </div>

          <div className="debug-card">
            <h3>{t('debug.section.chassis')}</h3>
            <div className="debug-rows">
              <div className="debug-row">
                <span>{t('debug.rideHeight')}</span>
                <span className="debug-value">
                  {formatOptionalRaw(debugChassis?.ride_height_mm, t('units.mm'))}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.suspensionFl')}</span>
                <span className="debug-value">{formatOptionalRaw(debugChassis?.suspension_fl)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.suspensionFr')}</span>
                <span className="debug-value">{formatOptionalRaw(debugChassis?.suspension_fr)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.suspensionRl')}</span>
                <span className="debug-value">{formatOptionalRaw(debugChassis?.suspension_rl)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.suspensionRr')}</span>
                <span className="debug-value">{formatOptionalRaw(debugChassis?.suspension_rr)}</span>
              </div>
            </div>
          </div>

          <div className="debug-card">
            <h3>{t('debug.section.gears')}</h3>
            <div className="debug-rows">
              <div className="debug-row">
                <span>{t('debug.gearRatio1')}</span>
                <span className="debug-value">{formatOptionalRatio(debugGears?.gear_ratio_1)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.gearRatio2')}</span>
                <span className="debug-value">{formatOptionalRatio(debugGears?.gear_ratio_2)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.gearRatio3')}</span>
                <span className="debug-value">{formatOptionalRatio(debugGears?.gear_ratio_3)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.gearRatio4')}</span>
                <span className="debug-value">{formatOptionalRatio(debugGears?.gear_ratio_4)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.gearRatio5')}</span>
                <span className="debug-value">{formatOptionalRatio(debugGears?.gear_ratio_5)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.gearRatio6')}</span>
                <span className="debug-value">{formatOptionalRatio(debugGears?.gear_ratio_6)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.gearRatio7')}</span>
                <span className="debug-value">{formatOptionalRatio(debugGears?.gear_ratio_7)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.gearRatio8')}</span>
                <span className="debug-value">{formatOptionalRatio(debugGears?.gear_ratio_8)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.gearRatioUnknown')}</span>
                <span className="debug-value">
                  {formatOptionalRatio(debugGears?.gear_ratio_unknown)}
                </span>
              </div>
            </div>
          </div>

          <div className="debug-card">
            <h3>{t('debug.section.dynamics')}</h3>
            <div className="debug-rows">
              <div className="debug-row">
                <span>{t('debug.posX')}</span>
                <span className="debug-value">{formatOptionalRaw(debugDynamics?.pos_x)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.posY')}</span>
                <span className="debug-value">{formatOptionalRaw(debugDynamics?.pos_y)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.posZ')}</span>
                <span className="debug-value">{formatOptionalRaw(debugDynamics?.pos_z)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.velX')}</span>
                <span className="debug-value">
                  {formatOptional(debugDynamics?.vel_x, numberFormats.speed, t('debug.unitMps'))}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.velY')}</span>
                <span className="debug-value">
                  {formatOptional(debugDynamics?.vel_y, numberFormats.speed, t('debug.unitMps'))}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.velZ')}</span>
                <span className="debug-value">
                  {formatOptional(debugDynamics?.vel_z, numberFormats.speed, t('debug.unitMps'))}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.angularVelX')}</span>
                <span className="debug-value">
                  {formatOptionalRaw(debugDynamics?.angular_vel_x, t('debug.unitRadS'))}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.angularVelY')}</span>
                <span className="debug-value">
                  {formatOptionalRaw(debugDynamics?.angular_vel_y, t('debug.unitRadS'))}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.angularVelZ')}</span>
                <span className="debug-value">
                  {formatOptionalRaw(debugDynamics?.angular_vel_z, t('debug.unitRadS'))}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.accelLong')}</span>
                <span className="debug-value">
                  {formatOptional(debugDynamics?.accel_long, numberFormats.speed, t('debug.unitMps2'))}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.accelLat')}</span>
                <span className="debug-value">
                  {formatOptional(debugDynamics?.accel_lat, numberFormats.speed, t('debug.unitMps2'))}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.yawRate')}</span>
                <span className="debug-value">
                  {formatOptional(debugDynamics?.yaw_rate, numberFormats.speed, t('debug.unitDegS'))}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.pitch')}</span>
                <span className="debug-value">
                  {formatOptional(debugDynamics?.pitch, numberFormats.speed, t('debug.unitDeg'))}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.roll')}</span>
                <span className="debug-value">
                  {formatOptional(debugDynamics?.roll, numberFormats.speed, t('debug.unitDeg'))}
                </span>
              </div>
              <div className="debug-row">
                <span>{t('debug.rotationYaw')}</span>
                <span className="debug-value">{formatOptionalRaw(debugDynamics?.rotation_yaw)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.rotationExtra')}</span>
                <span className="debug-value">{formatOptionalRaw(debugDynamics?.rotation_extra)}</span>
              </div>
            </div>
          </div>

          <div className="debug-card">
            <h3>{t('debug.section.flags')}</h3>
            <div className="debug-rows">
              <div className="debug-row">
                <span>{t('debug.flag8e')}</span>
                <span className="debug-value">{formatOptionalHex(debugFlags?.flags_8e)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.flag8f')}</span>
                <span className="debug-value">{formatOptionalHex(debugFlags?.flags_8f)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.flag93')}</span>
                <span className="debug-value">{formatOptionalHex(debugFlags?.flags_93)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.unknown94')}</span>
                <span className="debug-value">{formatOptionalRaw(debugFlags?.unknown_0x94)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.unknown98')}</span>
                <span className="debug-value">{formatOptionalRaw(debugFlags?.unknown_0x98)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.unknown9c')}</span>
                <span className="debug-value">{formatOptionalRaw(debugFlags?.unknown_0x9c)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.unknownA0')}</span>
                <span className="debug-value">{formatOptionalRaw(debugFlags?.unknown_0xa0)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.unknownD4')}</span>
                <span className="debug-value">{formatOptionalRaw(debugFlags?.unknown_0xd4)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.unknownD8')}</span>
                <span className="debug-value">{formatOptionalRaw(debugFlags?.unknown_0xd8)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.unknownDc')}</span>
                <span className="debug-value">{formatOptionalRaw(debugFlags?.unknown_0xdc)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.unknownE0')}</span>
                <span className="debug-value">{formatOptionalRaw(debugFlags?.unknown_0xe0)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.unknownE4')}</span>
                <span className="debug-value">{formatOptionalRaw(debugFlags?.unknown_0xe4)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.unknownE8')}</span>
                <span className="debug-value">{formatOptionalRaw(debugFlags?.unknown_0xe8)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.unknownEc')}</span>
                <span className="debug-value">{formatOptionalRaw(debugFlags?.unknown_0xec)}</span>
              </div>
              <div className="debug-row">
                <span>{t('debug.unknownF0')}</span>
                <span className="debug-value">{formatOptionalRaw(debugFlags?.unknown_0xf0)}</span>
              </div>
            </div>
          </div>
        </div>
      ) : (
        <div className="debug-raw-view">
          <div className="debug-raw-tabs">
            <button
              type="button"
              className={rawTab === 'encrypted' ? 'active' : ''}
              onClick={() => onRawTabChange('encrypted')}
            >
              {t('debug.rawEncrypted')}
            </button>
            <button
              type="button"
              className={rawTab === 'decrypted' ? 'active' : ''}
              onClick={() => onRawTabChange('decrypted')}
            >
              {t('debug.rawDecrypted')}
            </button>
          </div>
          <div className="debug-raw-meta">
            <div className="debug-row">
              <span>{t('debug.rawBytes')}</span>
              <span className="debug-value">
                {rawTab === 'encrypted'
                  ? formatOptionalInt(debugRaw?.encrypted_len)
                  : formatOptionalInt(debugRaw?.decrypted_len)}
              </span>
            </div>
            <div className="debug-row">
              <span>{t('debug.rawSourceIp')}</span>
              <span className="debug-value">{debugRaw?.source_ip ?? '—'}</span>
            </div>
            <div className="debug-row">
              <span>{t('debug.rawCapturedAt')}</span>
              <span className="debug-value">
                {formatOptionalTimestamp(debugRaw?.captured_at_ms)}
              </span>
            </div>
          </div>
          <textarea className="debug-raw-textarea mono" readOnly value={rawText} />
        </div>
      )}
    </section>
  )
}
