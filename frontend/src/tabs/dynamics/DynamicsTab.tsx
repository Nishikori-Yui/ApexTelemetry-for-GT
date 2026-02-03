import { useTranslation } from 'react-i18next'

export function DynamicsTab() {
  const { t } = useTranslation()
  return (
    <section className="panel tab-panel">
      <div className="panel-header">
        <h2>{t('tabs.dynamics')}</h2>
      </div>
      <div className="panel-body">
        <p className="muted">{t('placeholders.dynamics')}</p>
      </div>
    </section>
  )
}
