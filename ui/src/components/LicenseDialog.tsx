/**
 * Окно «Справка → Лицензия»: показывает текст лицензии Fontlord.
 */

import { useEffect, useState } from 'react';

import * as ipc from '../ipc';
import { useT } from '../i18n';
import { useStore } from '../store';

export function LicenseDialog() {
  const t = useT();
  const open = useStore((state) => state.licenseOpen);
  const closeLicense = useStore((state) => state.closeLicense);
  const [text, setText] = useState<string>('');

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    void ipc.appLicense().then((value) => {
      if (!cancelled) setText(value);
    });
    return () => {
      cancelled = true;
    };
  }, [open]);

  if (!open) return null;

  return (
    <div className="modal-backdrop" onClick={closeLicense}>
      <div className="modal license" onClick={(event) => event.stopPropagation()}>
        <h2>{t('license.title')}</h2>
        <p className="hint">{t('license.summary')}</p>
        <pre className="license-text">{text}</pre>

        <div className="modal-actions">
          <button type="button" className="primary" onClick={closeLicense}>
            {t('license.close')}
          </button>
        </div>
      </div>
    </div>
  );
}
