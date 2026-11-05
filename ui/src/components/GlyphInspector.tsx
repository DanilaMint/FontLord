/**
 * Инспектор глифа: метрики, слои (штрихи) и предпросмотр шрифта.
 */

import { useEffect, useMemo, useState } from 'react';

import * as ipc from '../ipc';
import { useStore } from '../store';

export function GlyphInspector() {
  const glyph = useStore((state) => state.glyph);
  const currentCodepoint = useStore((state) => state.currentCodepoint);
  const meta = useStore((state) => state.meta);
  const updateMetrics = useStore((state) => state.updateMetrics);
  const removeStroke = useStore((state) => state.removeStroke);
  const deleteGlyph = useStore((state) => state.deleteGlyph);

  const [advance, setAdvance] = useState(0);
  const [name, setName] = useState('');
  const [notes, setNotes] = useState('');

  useEffect(() => {
    setAdvance(glyph?.advanceWidth ?? 0);
    setName(glyph?.name ?? '');
    setNotes(glyph?.notes ?? '');
  }, [glyph]);

  const char = currentCodepoint !== null ? String.fromCodePoint(currentCodepoint) : '';

  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Глиф</h2>
        {currentCodepoint !== null && (
          <span className="badge">
            U+{currentCodepoint.toString(16).toUpperCase().padStart(4, '0')}
          </span>
        )}
      </div>

      {!glyph && (
        <div className="hint">
          {currentCodepoint !== null
            ? 'Глиф пуст. Нарисуйте первый штрих на холсте — глиф будет создан автоматически.'
            : 'Выберите символ в сетке слева, чтобы начать рисование.'}
        </div>
      )}

      {glyph && (
        <>
          <div className="glyph-head">
            <div className="glyph-preview-box">
              <span className="glyph-preview-char">{char}</span>
            </div>
            <div className="glyph-head-fields">
              <label className="field">
                <span>Имя</span>
                <input
                  className="input small"
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  onBlur={() => void updateMetrics(advance, name, notes)}
                />
              </label>
              <label className="field">
                <span>Ширина</span>
                <input
                  className="input small"
                  type="number"
                  min={0}
                  max={4000}
                  value={advance}
                  onChange={(event) => setAdvance(Number(event.target.value))}
                  onBlur={() => void updateMetrics(advance, name, notes)}
                />
              </label>
            </div>
          </div>

          <label className="field">
            <span>Заметки</span>
            <textarea
              className="input small"
              rows={2}
              value={notes}
              onChange={(event) => setNotes(event.target.value)}
              onBlur={() => void updateMetrics(advance, name, notes)}
            />
          </label>

          <div className="panel-header sub">
            <h3>Штрихи ({glyph.strokes.length})</h3>
            <button type="button" className="danger" onClick={() => void deleteGlyph()}>
              Удалить глиф
            </button>
          </div>

          {glyph.strokes.length === 0 && (
            <div className="hint">Штрихов пока нет. Рисуйте кистью на холсте.</div>
          )}

          <ul className="stroke-list">
            {glyph.strokes.map((stroke) => (
              <li key={stroke.id}>
                <span className={`stroke-kind kind-${stroke.kind}`} />
                <span className="stroke-info">
                  {stroke.kind} · {stroke.points.length} точек · {stroke.settings.width} ед.
                </span>
                <button type="button" onClick={() => void removeStroke(stroke.id)}>
                  ✕
                </button>
              </li>
            ))}
          </ul>

          {meta && (
            <div className="metrics-grid">
              <div>
                <span>upem</span>
                <em>{meta.unitsPerEm}</em>
              </div>
              <div>
                <span>asc/desc</span>
                <em>
                  {meta.ascender}/{meta.descender}
                </em>
              </div>
              <div>
                <span>x-height</span>
                <em>{meta.xHeight}</em>
              </div>
              <div>
                <span>cap</span>
                <em>{meta.capHeight}</em>
              </div>
            </div>
          )}
        </>
      )}
    </section>
  );
}

/**
 * Предпросмотр шрифта: собирает TTF на лету и показывает его в интерфейсе.
 */
export function FontPreview() {
  const project = useStore((state) => state.project);
  const glyph = useStore((state) => state.glyph);
  const [fontFamily, setFontFamily] = useState('FontlordPreview');

  const sample = useMemo(() => 'Aa Bb Cc 0123', []);

  useEffect(() => {
    let cancelled = false;
    const timer = window.setTimeout(async () => {
      try {
        const bytes = await ipc.fontPreviewBytes();
        if (cancelled) return;
        const buffer = new Uint8Array(bytes);
        const face = new FontFace(fontFamily, buffer.buffer as ArrayBuffer);
        await face.load();
        document.fonts.add(face);
        setFontFamily(`${fontFamily}`);
        // Принудительно обновляем отрисовку текста с новым шрифтом.
        setFontFamily((value) => value);
      } catch {
        /* шрифт ещё не собран — это нормально */
      }
    }, 400);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [project, glyph, fontFamily]);

  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Предпросмотр шрифта</h2>
      </div>
      <p className="preview-sample" style={{ fontFamily: `${fontFamily}, sans-serif` }}>
        {sample}
      </p>
      <p className="hint">
        Предпросмотр использует реальную сборку шрифта: контуры глифов, метрики и кернинг.
      </p>
    </section>
  );
}
