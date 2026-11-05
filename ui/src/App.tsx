/**
 * Корневой компонент приложения: заголовок, рабочая область и строка состояния.
 */

import { useEffect } from 'react';

import { FontPreview, GlyphInspector } from './components/GlyphInspector';
import { FontSettings } from './components/FontSettings';
import { NewProjectDialog } from './components/NewProjectDialog';
import { GlyphCanvas } from './components/GlyphCanvas';
import { BrushPanel } from './components/Inspector';
import { UnicodePanel } from './components/UnicodePanel';
import { useStore } from './store';

export function App() {
  const init = useStore((state) => state.init);
  const project = useStore((state) => state.project);
  const status = useStore((state) => state.status);
  const error = useStore((state) => state.error);
  const setError = useStore((state) => state.setError);
  const currentCodepoint = useStore((state) => state.currentCodepoint);

  const undo = useStore((state) => state.undo);
  const redo = useStore((state) => state.redo);
  const saveProject = useStore((state) => state.saveProject);
  const openProject = useStore((state) => state.openProject);
  const exportFont = useStore((state) => state.exportFont);
  const closeGlyph = useStore((state) => state.closeGlyph);
  const openNewProject = useStore((state) => state.openNewProject);

  useEffect(() => {
    void init();
  }, [init]);

  useEffect(() => {
    if (!error) return;
    const timer = window.setTimeout(() => setError(null), 6000);
    return () => window.clearTimeout(timer);
  }, [error, setError]);

  return (
    <div className="app">
      <header className="titlebar">
        <div className="brand">
          <span className="brand-mark">FL</span>
          <div>
            <strong>Fontlord</strong>
            <small>редактор шрифтов</small>
          </div>
        </div>

        <div className="project-info">
          <span className="project-name">
            {project?.meta.familyName ?? 'Без названия'} {project?.meta.styleName ?? ''}
          </span>
          <span className="project-path">
            {project?.path ?? 'не сохранён'}
            {project?.dirty ? ' •' : ''}
          </span>
        </div>

        <div className="toolbar">
          <button type="button" onClick={() => void undo()} disabled={!project?.canUndo}>
            ⟲ Отменить
          </button>
          <button type="button" onClick={() => void redo()} disabled={!project?.canRedo}>
            ⟳ Повторить
          </button>
          <button type="button" onClick={openNewProject}>
            Создать
          </button>
          <button type="button" onClick={() => void openProject()}>
            Открыть
          </button>
          <button type="button" onClick={() => void saveProject()}>
            Сохранить
          </button>
          <button type="button" className="primary" onClick={() => void exportFont()}>
            Экспорт TTF
          </button>
        </div>
      </header>

      <main className="layout">
        <aside className="sidebar left">
          <UnicodePanel />
        </aside>

        <section className="workspace">
          <div className="workspace-head">
            {currentCodepoint !== null ? (
              <>
                <span className="workspace-title">
                  {String.fromCodePoint(currentCodepoint)} — U+
                  {currentCodepoint.toString(16).toUpperCase().padStart(4, '0')}
                </span>
                <button type="button" onClick={closeGlyph}>
                  Закрыть
                </button>
              </>
            ) : (
              <span className="workspace-title muted">
                Выберите символ для редактирования
              </span>
            )}
          </div>
          <GlyphCanvas active={currentCodepoint !== null} />
        </section>

        <aside className="sidebar right">
          <FontPreview />
          <GlyphInspector />
          <BrushPanel />
          <FontSettings />
        </aside>
      </main>

      <NewProjectDialog />

      <footer className="statusbar">
        <span>{status}</span>
        {error && <span className="status-error">{error}</span>}
        <span className="spacer" />
        <span>
          Глифов: {project?.glyphCount ?? 0} · Единиц на em:{' '}
          {project?.meta.unitsPerEm ?? 1000}
        </span>
      </footer>
    </div>
  );
}
