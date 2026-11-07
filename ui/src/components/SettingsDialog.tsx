/**
 * Диалоги настроек: выбор языка интерфейса и темы оформления.
 */

import { LANGUAGES, type Language } from '../dict';
import { useT } from '../i18n';
import { useStore } from '../store';

/** Окно выбора языка интерфейса. */
export function LanguageDialog() {
  const open = useStore((state) => state.languageDialogOpen);
  const close = useStore((state) => state.closeLanguageDialog);
  const language = useStore((state) => state.language);
  const setLanguage = useStore((state) => state.setLanguage);
  const t = useT();

  if (!open) return null;

  return (
    <div className="modal-backdrop" onClick={close}>
      <div className="modal" onClick={(event) => event.stopPropagation()}>
        <h2>{t('settings.language.title')}</h2>
        <p className="hint">{t('settings.language.hint')}</p>

        <label className="field">
          <span>{t('settings.language.title')}</span>
          <select
            className="input"
            value={language}
            onChange={(event) => setLanguage(event.target.value as Language)}
          >
            {LANGUAGES.map((item) => (
              <option key={item.id} value={item.id}>
                {item.title}
              </option>
            ))}
          </select>
        </label>

        <div className="modal-actions">
          <button type="button" className="primary" onClick={close}>
            {t('settings.close')}
          </button>
        </div>
      </div>
    </div>
  );
}
