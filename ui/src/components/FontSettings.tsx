/**
 * Панель параметров шрифта: имя семейства, начертание и вертикальные метрики.
 */

import { useEffect, useState } from 'react';

import * as ipc from '../ipc';
import { useStore } from '../store';

export function FontSettings() {
  const meta = useStore((state) => state.meta);
  const refreshProject = useStore((state) => state.refreshProject);
  const setStatus = useStore((state) => state.setStatus);

  const [family, setFamily] = useState('');
  const [style, setStyle] = useState('');
  const [version, setVersion] = useState('');
  const [upem, setUpem] = useState(1000);
  const [ascender, setAscender] = useState(800);
  const [descender, setDescender] = useState(-200);
  const [xHeight, setXHeight] = useState(500);
  const [capHeight, setCapHeight] = useState(700);

  useEffect(() => {
    if (!meta) return;
    setFamily(meta.familyName);
    setStyle(meta.styleName);
    setVersion(meta.version);
    setUpem(meta.unitsPerEm);
    setAscender(meta.ascender);
    setDescender(meta.descender);
    setXHeight(meta.xHeight);
    setCapHeight(meta.capHeight);
  }, [meta]);

  /** Сохраняет изменённые метрики в проекте. */
  const save = async () => {
    if (!meta) return;
    try {
      await ipc.projectSetMeta({
        ...meta,
        familyName: family,
        styleName: style,
        version,
        unitsPerEm: upem,
        ascender,
        descender,
        xHeight,
        capHeight,
      });
      await refreshProject();
      setStatus('Параметры шрифта обновлены');
    } catch (error) {
      setStatus(`Ошибка: ${String(error)}`);
    }
  };

  if (!meta) return null;

  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Шрифт</h2>
      </div>

      <div className="meta-grid">
        <label className="field">
          <span>Семейство</span>
          <input
            className="input small"
            value={family}
            onChange={(event) => setFamily(event.target.value)}
            onBlur={() => void save()}
          />
        </label>
        <label className="field">
          <span>Начертание</span>
          <input
            className="input small"
            value={style}
            onChange={(event) => setStyle(event.target.value)}
            onBlur={() => void save()}
          />
        </label>
        <label className="field">
          <span>Версия</span>
          <input
            className="input small"
            value={version}
            onChange={(event) => setVersion(event.target.value)}
            onBlur={() => void save()}
          />
        </label>
        <label className="field">
          <span>Единиц на em</span>
          <input
            className="input small"
            type="number"
            min={16}
            max={16384}
            step={1}
            value={upem}
            onChange={(event) => setUpem(Number(event.target.value))}
            onBlur={() => void save()}
          />
        </label>
        <label className="field">
          <span>Ascender</span>
          <input
            className="input small"
            type="number"
            value={ascender}
            onChange={(event) => setAscender(Number(event.target.value))}
            onBlur={() => void save()}
          />
        </label>
        <label className="field">
          <span>Descender</span>
          <input
            className="input small"
            type="number"
            value={descender}
            onChange={(event) => setDescender(Number(event.target.value))}
            onBlur={() => void save()}
          />
        </label>
        <label className="field">
          <span>x-height</span>
          <input
            className="input small"
            type="number"
            value={xHeight}
            onChange={(event) => setXHeight(Number(event.target.value))}
            onBlur={() => void save()}
          />
        </label>
        <label className="field">
          <span>Cap-height</span>
          <input
            className="input small"
            type="number"
            value={capHeight}
            onChange={(event) => setCapHeight(Number(event.target.value))}
            onBlur={() => void save()}
          />
        </label>
      </div>
    </section>
  );
}
