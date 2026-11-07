/**
 * Модальное окно создания нового проекта.
 *
 * Используется вместо `window.prompt`, который недоступен в webview Tauri.
 */

import { useEffect, useRef, useState } from 'react';

import { useT } from '../i18n';
import { useStore } from '../store';

export function NewProjectDialog() {
  const t = useT();
  const open = useStore((state) => state.newProjectOpen);
  const newProject = useStore((state) => state.newProject);
  const closeNewProject = useStore((state) => state.closeNewProject);

  const [family, setFamily] = useState('My Font');
  const [style, setStyle] = useState('Regular');
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setFamily('My Font');
      setStyle('Regular');
      // Фокус на поле после появления окна.
      window.setTimeout(() => inputRef.current?.focus(), 30);
    }
  }, [open]);

  if (!open) return null;

  const submit = () => {
    void newProject(family.trim() || 'My Font', style.trim() || 'Regular');
  };

  return (
    <div className="modal-backdrop" onClick={closeNewProject}>
      <div className="modal" onClick={(event) => event.stopPropagation()}>
        <h2>{t('new.title')}</h2>
        <p className="hint">{t('new.hint')}</p>

        <label className="field">
          <span>{t('new.family')}</span>
          <input
            ref={inputRef}
            className="input"
            value={family}
            onChange={(event) => setFamily(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter') submit();
            }}
          />
        </label>

        <label className="field">
          <span>{t('new.style')}</span>
          <input
            className="input"
            value={style}
            onChange={(event) => setStyle(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter') submit();
            }}
          />
        </label>

        <div className="modal-actions">
          <button type="button" onClick={closeNewProject}>
            {t('new.cancel')}
          </button>
          <button type="button" className="primary" onClick={submit}>
            {t('new.create')}
          </button>
        </div>
      </div>
    </div>
  );
}
