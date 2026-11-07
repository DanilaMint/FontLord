/**
 * Окно выбора формата экспорта шрифта.
 *
 * Fontlord собирает шрифт в четырёх форматах: TrueType, OpenType с
 * контурами CFF и веб-форматы WOFF/WOFF2.
 */

import { useStore } from '../store';
import type { FontFormat } from '../types';

/** Описание форматов, доступных для экспорта. */
const FORMATS: { format: FontFormat; title: string; hint: string }[] = [
  {
    format: 'ttf',
    title: 'TrueType (.ttf)',
    hint: 'Классический формат шрифтов, поддерживается везде.',
  },
  {
    format: 'otf',
    title: 'OpenType (.otf)',
    hint: 'Контуры CFF вместо квадратичных кривых TrueType.',
  },
  {
    format: 'woff',
    title: 'WOFF (.woff)',
    hint: 'Веб-формат: те же таблицы, сжатие zlib.',
  },
  {
    format: 'woff2',
    title: 'WOFF2 (.woff2)',
    hint: 'Веб-формат со сжатием Brotli — самый компактный.',
  },
];

export function ExportDialog() {
  const open = useStore((state) => state.exportOpen);
  const closeExport = useStore((state) => state.closeExport);
  const exportFont = useStore((state) => state.exportFont);

  if (!open) return null;

  return (
    <div className="modal-backdrop" onClick={closeExport}>
      <div className="modal" onClick={(event) => event.stopPropagation()}>
        <h2>Экспорт шрифта</h2>
        <p className="hint">Выберите формат собираемого файла.</p>

        <div className="format-list">
          {FORMATS.map((item) => (
            <button
              key={item.format}
              type="button"
              className="format-item"
              onClick={() => void exportFont(item.format)}
            >
              <strong>{item.title}</strong>
              <small>{item.hint}</small>
            </button>
          ))}
        </div>

        <div className="modal-actions">
          <button type="button" onClick={closeExport}>
            Отмена
          </button>
        </div>
      </div>
    </div>
  );
}
