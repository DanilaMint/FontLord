/**
 * Окно выбора формата экспорта шрифта.
 *
 * Fontlord собирает шрифт в четырёх форматах: TrueType, OpenType с
 * контурами CFF и веб-форматы WOFF/WOFF2.
 */

import { useT } from '../i18n';
import { useStore } from '../store';
import type { FontFormat } from '../types';

/** Описание форматов, доступных для экспорта. */
const FORMATS: { format: FontFormat; title: string; hint: string }[] = [
  { format: 'ttf', title: 'TrueType (.ttf)', hint: 'export.ttf' },
  { format: 'otf', title: 'OpenType (.otf)', hint: 'export.otf' },
  { format: 'woff', title: 'WOFF (.woff)', hint: 'export.woff' },
  { format: 'woff2', title: 'WOFF2 (.woff2)', hint: 'export.woff2' },
];

export function ExportDialog() {
  const t = useT();
  const open = useStore((state) => state.exportOpen);
  const closeExport = useStore((state) => state.closeExport);
  const exportFont = useStore((state) => state.exportFont);

  if (!open) return null;

  return (
    <div className="modal-backdrop" onClick={closeExport}>
      <div className="modal" onClick={(event) => event.stopPropagation()}>
        <h2>{t('export.title')}</h2>
        <p className="hint">{t('export.hint')}</p>

        <div className="format-list">
          {FORMATS.map((item) => (
            <button
              key={item.format}
              type="button"
              className="format-item"
              onClick={() => void exportFont(item.format)}
            >
              <strong>{item.title}</strong>
              <small>{t(item.hint)}</small>
            </button>
          ))}
        </div>

        <div className="modal-actions">
          <button type="button" onClick={closeExport}>
            {t('export.cancel')}
          </button>
        </div>
      </div>
    </div>
  );
}
