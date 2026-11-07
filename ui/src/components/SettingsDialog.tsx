/**
 * Диалоги настроек: выбор языка интерфейса и темы оформления.
 */

import { LANGUAGES, type Language } from '../dict';
import { useT } from '../i18n';
import { useStore } from '../store';
import type { Theme } from '../theme';

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

/** Окно выбора темы оформления. */
export function ThemeDialog() {
  const open = useStore((state) => state.themeDialogOpen);
  const close = useStore((state) => state.closeThemeDialog);
  const theme = useStore((state) => state.theme);
  const setTheme = useStore((state) => state.setTheme);
  const t = useT();

  if (!open) return null;

  return (
    <div className="modal-backdrop" onClick={close}>
      <div className="modal" onClick={(event) => event.stopPropagation()}>
        <h2>{t('settings.theme.title')}</h2>
        <p className="hint">{t('settings.theme.hint')}</p>

        <label className="field">
          <span>{t('settings.theme.title')}</span>
          <select
            className="input"
            value={theme}
            onChange={(event) => setTheme(event.target.value as Theme)}
          >
            <option value="dark">{t('theme.dark')}</option>
            <option value="light">{t('theme.light')}</option>
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
