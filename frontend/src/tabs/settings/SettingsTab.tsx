import type { Dispatch, SetStateAction } from 'react'
import { useTranslation } from 'react-i18next'
import { UNIT_KEYS } from '../../utils/units'
import type {
  DetectStatus,
  FuelUnit,
  PressureUnit,
  SpeedUnit,
  TempUnit,
  UiLog,
} from '../../types'

type SettingsTabProps = {
  telemetryActive: boolean
  detectStatus: DetectStatus
  detectIp: string | null
  ps5Input: string
  setPs5Input: (value: string) => void
  onStartAutoDetect: () => void
  onManualBind: () => void
  demoActive: boolean
  demoPath: string | null
  demoPending: boolean
  demoError: string | null
  onToggleDemo: () => void
  selectedLang: string
  onApplyLanguage: (value: string) => void
  speedUnit: SpeedUnit
  tempUnit: TempUnit
  pressureUnit: PressureUnit
  fuelUnit: FuelUnit
  onUpdateUnit: <T extends string>(
    key: string,
    value: string,
    setter: (next: T) => void,
  ) => void
  setSpeedUnit: (next: SpeedUnit) => void
  setTempUnit: (next: TempUnit) => void
  setPressureUnit: (next: PressureUnit) => void
  setFuelUnit: (next: FuelUnit) => void
  uiDebugEnabled: boolean
  setUiDebugEnabled: Dispatch<SetStateAction<boolean>>
  uiLogs: UiLog[]
  onClearUiLogs: () => void
}

export function SettingsTab({
  telemetryActive,
  detectStatus,
  detectIp,
  ps5Input,
  setPs5Input,
  onStartAutoDetect,
  onManualBind,
  demoActive,
  demoPath,
  demoPending,
  demoError,
  onToggleDemo,
  selectedLang,
  onApplyLanguage,
  speedUnit,
  tempUnit,
  pressureUnit,
  fuelUnit,
  onUpdateUnit,
  setSpeedUnit,
  setTempUnit,
  setPressureUnit,
  setFuelUnit,
  uiDebugEnabled,
  setUiDebugEnabled,
  uiLogs,
  onClearUiLogs,
}: SettingsTabProps) {
  const { t } = useTranslation()
  const ps5StatusLabel =
    detectStatus === 'pending'
      ? t('detect.status.pending')
      : telemetryActive
        ? t('telemetry.status.active')
        : t('telemetry.status.idle')

  const languages = [
    { value: 'en', label: t('languages.en') },
    { value: 'zh-CN', label: t('languages.zh-CN') },
    { value: 'zh-TW', label: t('languages.zh-TW') },
    { value: 'ja', label: t('languages.ja') },
  ]

  return (
    <section className="panel tab-panel settings-panel">
      <div className="panel-header">
        <h2>{t('settings.title')}</h2>
      </div>

      <div className="settings-gridbox">
        <div className="settings-section">
          <h3>{t('ps5.title')}</h3>
          <div className="settings-row">
            <span className="settings-label">{t('ps5.status')}</span>
            <span className={`connection-status ${telemetryActive ? 'green' : 'red'}`}>
              {ps5StatusLabel}
            </span>
          </div>

          <div className="settings-row input-row">
            <label>
              {t('ps5.ps5Ip')}
              <div className="ip-input-group">
                <input
                  value={ps5Input}
                  onChange={(event) => setPs5Input(event.target.value)}
                  placeholder={detectIp || t('ps5.placeholder')}
                  className="settings-input"
                />
                {detectIp && ps5Input !== detectIp && (
                  <button
                    className="action-btn-small"
                    onClick={() => setPs5Input(detectIp)}
                    title={`Click to fill detected IP: ${detectIp}`}
                  >
                    Fill
                  </button>
                )}
              </div>
            </label>
          </div>

          <div className="settings-actions">
            <button
              type="button"
              className="action-btn"
              disabled={detectStatus === 'pending'}
              onClick={onStartAutoDetect}
            >
              {detectStatus === 'pending' ? 'Scanning...' : t('ps5.autoDetect')}
            </button>
            <button type="button" className="action-btn primary" onClick={onManualBind}>
              {t('ps5.manualBind')}
            </button>
          </div>
        </div>

        <div className="settings-section">
          <h3>{t('demo.title')}</h3>
          <div className="settings-row">
            <span className="settings-label">{t('demo.status')}</span>
            <span className={`connection-status ${demoActive ? 'green' : 'red'}`}>
              {demoActive ? t('demo.active') : t('demo.inactive')}
            </span>
          </div>
          <div className="settings-row">
            <span className="settings-label">{t('demo.source')}</span>
            <span className="settings-value">{demoPath ?? t('demo.defaultPath')}</span>
          </div>
          {demoError && <div className="settings-hint error">{demoError}</div>}
          <div className="settings-actions">
            <button
              type="button"
              className={`action-btn ${demoActive ? '' : 'primary'}`}
              disabled={demoPending}
              onClick={onToggleDemo}
            >
              {demoPending
                ? demoActive
                  ? t('demo.stopping')
                  : t('demo.starting')
                : demoActive
                  ? t('demo.stop')
                  : t('demo.start')}
            </button>
          </div>
        </div>

        <div className="settings-section">
          <h3>{t('settings.language')}</h3>
          <select
            className="settings-select full-width"
            value={selectedLang}
            onChange={(event) => onApplyLanguage(event.target.value)}
          >
            {languages.map((lang) => (
              <option key={lang.value} value={lang.value}>
                {lang.label}
              </option>
            ))}
          </select>
        </div>

        <div className="settings-section">
          <h3>{t('settings.units')}</h3>
          <div className="settings-fields-grid">
            <label className="settings-field">
              <span>{t('settings.unitSpeed')}</span>
              <select
                className="settings-select"
                value={speedUnit}
                onChange={(e) => onUpdateUnit(UNIT_KEYS.speed, e.target.value, setSpeedUnit)}
              >
                <option value="kph">{t('units.kph')}</option>
                <option value="mph">{t('units.mph')}</option>
              </select>
            </label>
            <label className="settings-field">
              <span>{t('settings.unitTemp')}</span>
              <select
                className="settings-select"
                value={tempUnit}
                onChange={(e) => onUpdateUnit(UNIT_KEYS.temp, e.target.value, setTempUnit)}
              >
                <option value="c">{t('units.c')}</option>
                <option value="f">{t('units.f')}</option>
              </select>
            </label>
            <label className="settings-field">
              <span>{t('settings.unitPressure')}</span>
              <select
                className="settings-select"
                value={pressureUnit}
                onChange={(e) =>
                  onUpdateUnit(UNIT_KEYS.pressure, e.target.value, setPressureUnit)
                }
              >
                <option value="kpa">{t('units.kpa')}</option>
                <option value="psi">{t('units.psi')}</option>
              </select>
            </label>
            <label className="settings-field">
              <span>{t('settings.unitFuel')}</span>
              <select
                className="settings-select"
                value={fuelUnit}
                onChange={(e) => onUpdateUnit(UNIT_KEYS.fuel, e.target.value, setFuelUnit)}
              >
                <option value="l">{t('units.l')}</option>
                <option value="gal">{t('units.gal')}</option>
              </select>
            </label>
          </div>
        </div>

        <div className="settings-section settings-section-wide">
          <h3>{t('uiDebug.title')}</h3>
          <div className="settings-row">
            <span className="settings-label">{t('uiDebug.mode')}</span>
            <button
              type="button"
              className={`action-btn-small${uiDebugEnabled ? ' active' : ''}`}
              onClick={() => setUiDebugEnabled((prev) => !prev)}
            >
              {uiDebugEnabled ? t('uiDebug.on') : t('uiDebug.off')}
            </button>
          </div>
          {uiDebugEnabled && (
            <>
              <div className="settings-actions">
                <button type="button" className="action-btn" onClick={onClearUiLogs}>
                  {t('uiDebug.clear')}
                </button>
              </div>
              <div className="debug-log">
                {uiLogs.length === 0 ? (
                  <div className="debug-log-empty">{t('uiDebug.empty')}</div>
                ) : (
                  uiLogs.map((log, index) => (
                    <div key={`${log.at}-${index}`} className={`debug-log-entry ${log.level}`}>
                      <span className="debug-log-time">
                        {new Date(log.at).toLocaleTimeString()}
                      </span>
                      <span className="debug-log-level">{log.level}</span>
                      <span className="debug-log-message">{log.message}</span>
                    </div>
                  ))
                )}
              </div>
            </>
          )}
        </div>
      </div>
    </section>
  )
}
