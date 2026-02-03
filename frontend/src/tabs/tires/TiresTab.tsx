import { useTranslation } from 'react-i18next'

export function TiresTab() {
  const { t } = useTranslation()
  return (
    <section className="panel tab-panel">
      <div className="panel-header">
        <h2>{t('tabs.tires')}</h2>
      </div>
      <div className="panel-body">
        <p className="muted">{t('placeholders.tires')}</p>
      </div>
    </section>
  )
}
