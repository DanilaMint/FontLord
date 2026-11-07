/**
 * Панель параметров шрифта: имя семейства, начертание и вертикальные метрики.
 */

import { useEffect, useState } from 'react';

import * as ipc from '../ipc';
import { useT } from '../i18n';
import { useStore } from '../store';

export function FontSettings() {
  const t = useT();
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
      setStatus(t('font.saved'));
    } catch (error) {
      setStatus(t('status.error', { message: String(error) }));
    }
  };

  if (!meta) return null;

  return (
    <section className="panel">
      <div className="panel-header">
        <h2>{t('font.title')}</h2>
      </div>

      <div className="meta-grid">
        <label className="field">
          <span>{t('font.family')}</span>
          <input
            className="input small"
            value={family}
            onChange={(event) => setFamily(event.target.value)}
            onBlur={() => void save()}
          />
        </label>
        <label className="field">
          <span>{t('font.style')}</span>
          <input
            className="input small"
            value={style}
            onChange={(event) => setStyle(event.target.value)}
            onBlur={() => void save()}
          />
        </label>
        <label className="field">
          <span>{t('font.version')}</span>
          <input
            className="input small"
            value={version}
            onChange={(event) => setVersion(event.target.value)}
            onBlur={() => void save()}
          />
        </label>
        <label className="field">
          <span>{t('font.upem')}</span>
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
          <span>{t('font.ascender')}</span>
          <input
            className="input small"
            type="number"
            value={ascender}
            onChange={(event) => setAscender(Number(event.target.value))}
            onBlur={() => void save()}
          />
        </label>
        <label className="field">
          <span>{t('font.descender')}</span>
          <input
            className="input small"
            type="number"
            value={descender}
            onChange={(event) => setDescender(Number(event.target.value))}
            onBlur={() => void save()}
          />
        </label>
        <label className="field">
          <span>{t('font.xHeight')}</span>
          <input
            className="input small"
            type="number"
            value={xHeight}
            onChange={(event) => setXHeight(Number(event.target.value))}
            onBlur={() => void save()}
          />
        </label>
        <label className="field">
          <span>{t('font.capHeight')}</span>
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
