/**
 * Корневой компонент приложения: меню, рабочая область и строка состояния.
 */

import { useEffect } from 'react';

import { AboutDialog } from './components/AboutDialog';
import { ExportDialog } from './components/ExportDialog';
import { FontSettings } from './components/FontSettings';
import { FontPreview, GlyphInspector } from './components/GlyphInspector';
import { GlyphCanvas } from './components/GlyphCanvas';
import { BrushPanel } from './components/Inspector';
import { LicenseDialog } from './components/LicenseDialog';
import { MenuBar } from './components/MenuBar';
import { LanguageDialog, ThemeDialog } from './components/SettingsDialog';
import { NewProjectDialog } from './components/NewProjectDialog';
import { UnicodePanel } from './components/UnicodePanel';
import * as ipc from './ipc';
import { useT } from './i18n';
import { useStore } from './store';

/** Формирует заголовок окна по состоянию проекта. */
function windowTitle(path: string | null, family: string, style: string): string {
  if (path) {
    const name = path.split(/[\\/]/).pop() ?? path;
    return `Fontlord — ${name}`;
  }
  const base = `${family} ${style}`.trim();
  return `Fontlord — ${base || 'Новый проект'}.fontlord`;
}

export function App() {
  const t = useT();
  const init = useStore((state) => state.init);
  const project = useStore((state) => state.project);
  const status = useStore((state) => state.status);
  const error = useStore((state) => state.error);
  const setError = useStore((state) => state.setError);
  const currentCodepoint = useStore((state) => state.currentCodepoint);
  const glyph = useStore((state) => state.glyph);
  const leftPanelVisible = useStore((state) => state.leftPanelVisible);
  const theme = useStore((state) => state.theme);
  const language = useStore((state) => state.language);

  const undo = useStore((state) => state.undo);
  const redo = useStore((state) => state.redo);
  const saveProject = useStore((state) => state.saveProject);
  const openProject = useStore((state) => state.openProject);
  const openExport = useStore((state) => state.openExport);
  const closeGlyph = useStore((state) => state.closeGlyph);
  const openNewProject = useStore((state) => state.openNewProject);
  const openAbout = useStore((state) => state.openAbout);
  const toggleLeftPanel = useStore((state) => state.toggleLeftPanel);
  const zoomIn = useStore((state) => state.zoomIn);
  const zoomOut = useStore((state) => state.zoomOut);
  const zoomReset = useStore((state) => state.zoomReset);
  const setBrushKind = useStore((state) => state.setBrushKind);
  const deleteGlyph = useStore((state) => state.deleteGlyph);
  const cursorMode = useStore((state) => state.cursorMode);
  const toggleCursorMode = useStore((state) => state.toggleCursorMode);
  const selectedStrokeIds = useStore((state) => state.selectedStrokeIds);
  const clearSelection = useStore((state) => state.clearSelection);
  const deleteSelectedStrokes = useStore((state) => state.deleteSelectedStrokes);
  const copySelection = useStore((state) => state.copySelection);
  const cutSelection = useStore((state) => state.cutSelection);
  const pasteClipboard = useStore((state) => state.pasteClipboard);
  const selectAllStrokes = useStore((state) => state.selectAllStrokes);

  useEffect(() => {
    void init();
  }, [init]);

  // Тема и язык применяются к документу сразу при запуске.
  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  useEffect(() => {
    document.documentElement.lang = language;
  }, [language]);

  useEffect(() => {
    if (!error) return;
    const timer = window.setTimeout(() => setError(null), 6000);
    return () => window.clearTimeout(timer);
  }, [error, setError]);

  // Заголовок окна следует за проектом.
  useEffect(() => {
    if (!project) return;
    const title = windowTitle(
      project.path,
      project.meta.familyName,
      project.meta.styleName,
    );
    document.title = title;
    void ipc.setWindowTitle(title).catch(() => undefined);
  }, [project]);

  // Горячие клавиши.
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const typing =
        target !== null &&
        (target.tagName === 'INPUT' ||
          target.tagName === 'TEXTAREA' ||
          target.isContentEditable);
      const ctrl = event.ctrlKey || event.metaKey;

      if (ctrl) {
        // В текстовых полях сочетания работы с буфером обмена остаются
        // системными: выделение текста важнее выделения штрихов.
        if (typing && ['a', 'c', 'x', 'v'].includes(event.key.toLowerCase())) {
          return;
        }
        switch (event.key.toLowerCase()) {
          case 'n':
            event.preventDefault();
            openNewProject();
            return;
          case 'o':
            event.preventDefault();
            void openProject();
            return;
          case 's':
            event.preventDefault();
            void saveProject(undefined, event.shiftKey);
            return;
          case 'e':
            event.preventDefault();
            openExport();
            return;
          case 'z':
            event.preventDefault();
            if (event.shiftKey) void redo();
            else void undo();
            return;
          case 'y':
            event.preventDefault();
            void redo();
            return;
          case 'q':
            event.preventDefault();
            void ipc.appExit();
            return;
          case 'b':
            event.preventDefault();
            toggleLeftPanel();
            return;
          case 'c':
            if (selectedStrokeIds.length > 0) {
              event.preventDefault();
              copySelection();
            }
            return;
          case 'x':
            if (selectedStrokeIds.length > 0) {
              event.preventDefault();
              void cutSelection();
            }
            return;
          case 'v':
            event.preventDefault();
            void pasteClipboard();
            return;
          case 'a':
            if (glyph) {
              event.preventDefault();
              selectAllStrokes();
            }
            return;
          case '=':
          case '+':
            event.preventDefault();
            zoomIn();
            return;
          case '-':
            event.preventDefault();
            zoomOut();
            return;
          case '0':
            event.preventDefault();
            zoomReset();
            return;
          default:
            return;
        }
      }

      if (typing) return;

      switch (event.key) {
        case 'Delete':
        case 'Backspace':
          if (selectedStrokeIds.length > 0) {
            event.preventDefault();
            void deleteSelectedStrokes();
          } else if (glyph) {
            event.preventDefault();
            void deleteGlyph();
          }
          return;
        case 'Escape':
          if (selectedStrokeIds.length > 0) clearSelection();
          else closeGlyph();
          return;
        case 'F1':
          event.preventDefault();
          openAbout();
          return;
        default:
          break;
      }

      switch (event.key.toLowerCase()) {
        case 'b':
          setBrushKind('pen');
          break;
        case 'm':
          setBrushKind('marker');
          break;
        case 'n':
          setBrushKind('broadNib');
          break;
        case 's':
          setBrushKind('soft');
          break;
        case 'e':
          setBrushKind('eraser');
          break;
        case 'v':
          toggleCursorMode();
          break;
        default:
          break;
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [
    clearSelection,
    closeGlyph,
    copySelection,
    cursorMode,
    cutSelection,
    deleteGlyph,
    deleteSelectedStrokes,
    glyph,
    pasteClipboard,
    selectAllStrokes,
    selectedStrokeIds,
    toggleCursorMode,
    openAbout,
    openExport,
    openNewProject,
    openProject,
    redo,
    saveProject,
    setBrushKind,
    toggleLeftPanel,
    undo,
    zoomIn,
    zoomOut,
    zoomReset,
  ]);

  return (
    <div className={`app${leftPanelVisible ? '' : ' no-left-panel'}`}>
      <MenuBar />

      <header className="titlebar">
        <div className="brand">
          <span className="brand-mark">FL</span>
          <div>
            <strong>Fontlord</strong>
            <small>{t('toolbar.subtitle')}</small>
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
            ⟲ {t('toolbar.undo')}
          </button>
          <button type="button" onClick={() => void redo()} disabled={!project?.canRedo}>
            ⟳ {t('toolbar.redo')}
          </button>
          <button type="button" onClick={openNewProject}>
            {t('toolbar.create')}
          </button>
          <button type="button" onClick={() => void openProject()}>
            {t('toolbar.open')}
          </button>
          <button type="button" onClick={() => void saveProject()}>
            {t('toolbar.save')}
          </button>
          <button type="button" className="primary" onClick={openExport}>
            {t('toolbar.export')}
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
                  {t('workspace.close')}
                </button>
              </>
            ) : (
              <span className="workspace-title muted">
                {t('workspace.selectGlyph')}
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
      <ExportDialog />
      <AboutDialog />
      <LicenseDialog />
      <LanguageDialog />
      <ThemeDialog />

      <footer className="statusbar">
        <span>{status}</span>
        {error && <span className="status-error">{error}</span>}
        <span className="spacer" />
        <span>
          {t('status.glyphs', {
            glyphs: project?.glyphCount ?? 0,
            upem: project?.meta.unitsPerEm ?? 1000,
          })}
        </span>
      </footer>
    </div>
  );
}
