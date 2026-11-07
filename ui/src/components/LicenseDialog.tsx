/**
 * Окно «Справка → Лицензия»: показывает текст лицензии Fontlord.
 */

import { useEffect, useState } from 'react';

import * as ipc from '../ipc';
import { useStore } from '../store';

export function LicenseDialog() {
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
        <h2>Лицензия Fontlord</h2>
        <p className="hint">
          Код программы можно свободно читать, изменять и распространять.
          Программу как таковую продавать нельзя, зато декомпиляция и
          исследование кода прямо разрешены. Все шрифты, созданные в Fontlord,
          принадлежат вам: их можно свободно использовать и продавать.
        </p>
        <pre className="license-text">{text}</pre>

        <div className="modal-actions">
          <button type="button" className="primary" onClick={closeLicense}>
            Закрыть
          </button>
        </div>
      </div>
    </div>
  );
}
