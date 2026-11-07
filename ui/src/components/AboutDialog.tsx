/**
 * Окно «О программе».
 */

import { getVersion } from '@tauri-apps/api/app';
import { useEffect, useState } from 'react';

import { useT } from '../i18n';
import { useStore } from '../store';

export function AboutDialog() {
  const t = useT();
  const [version, setVersion] = useState('');
  const open = useStore((state) => state.aboutOpen);
  const closeAbout = useStore((state) => state.closeAbout);

  useEffect(() => {
    let cancelled = false;
    void getVersion()
      .then((value) => {
        if (!cancelled) setVersion(value);
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, []);

  if (!open) return null;

  return (
    <div className="modal-backdrop" onClick={closeAbout}>
      <div className="modal about" onClick={(event) => event.stopPropagation()}>
        <div className="about-head">
          <span className="brand-mark large">FL</span>
          <div>
            <h2>
              {t('about.title')}
              {version ? ` ${version}` : ''}
            </h2>
            <p className="hint">{t('about.subtitle')}</p>
          </div>
        </div>

        <p className="hint">{t('about.text')}</p>

        <table className="about-table">
          <tbody>
            <tr>
              <td>Ctrl+N / Ctrl+O / Ctrl+S</td>
              <td>{t('about.hotkey.project')}</td>
            </tr>
            <tr>
              <td>Ctrl+E</td>
              <td>{t('about.hotkey.export')}</td>
            </tr>
            <tr>
              <td>Ctrl+Z / Ctrl+Y</td>
              <td>{t('about.hotkey.history')}</td>
            </tr>
            <tr>
              <td>B / M / N / S / E</td>
              <td>{t('about.hotkey.brushes')}</td>
            </tr>
            <tr>
              <td>Ctrl+0 / Ctrl++ / Ctrl+−</td>
              <td>{t('about.hotkey.zoom')}</td>
            </tr>
            <tr>
              <td>Del / Esc</td>
              <td>{t('about.hotkey.glyph')}</td>
            </tr>
            <tr>
              <td>V</td>
              <td>{t('about.hotkey.cursor')}</td>
            </tr>
          </tbody>
        </table>

        <div className="modal-actions">
          <button type="button" className="primary" onClick={closeAbout}>
            {t('about.close')}
          </button>
        </div>
      </div>
    </div>
  );
}
