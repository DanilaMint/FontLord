/**
 * Глобальное состояние приложения (zustand).
 *
 * Хранилище отвечает за загрузку данных (Unicode-индекс, проект, глифы) и
 * за вызовы IPC. Компоненты только читают состояние и вызывают действия.
 */

import { create } from 'zustand';

import { loadLanguage, saveLanguage, translate, type Language } from './dict';
import { loadTheme, saveTheme, type Theme } from './theme';
import * as ipc from './ipc';
import type {
  BlockInfo,
  BrushKind,
  BrushPreset,
  BrushSettings,
  CodepointInfo,
  FontFormat,
  FontMeta,
  Glyph,
  ProjectState,
  Stroke,
} from './types';

/** Размер страницы сетки символов. */
const GRID_PAGE = 1500;

/** Начальные параметры кисти. */
const DEFAULT_BRUSH: BrushSettings = {
  width: 60,
  angle: Math.PI / 4,
  minWidthRatio: 0.35,
  pressureGamma: 1,
  taperIn: 0,
  taperOut: 0,
  hardness: 0.8,
};

interface AppState {
  /* Проект */
  project: ProjectState | null;
  meta: FontMeta | null;

  /* Сетка Unicode */
  blocks: BlockInfo[];
  selectedBlock: BlockInfo | null;
  gridItems: CodepointInfo[];
  gridExhausted: boolean;
  searchQuery: string;
  searchResult: { items: CodepointInfo[]; total: number } | null;
  searching: boolean;

  /* Редактор */
  currentCodepoint: number | null;
  glyph: Glyph | null;
  brushKind: BrushKind;
  brushSettings: BrushSettings;
  presets: BrushPreset[];
  nextStrokeId: number;

  /* Режим курсора и выделение */
  cursorMode: boolean;
  selectedStrokeIds: number[];
  clipboard: Stroke[];

  /* Служебное */
  status: string;
  error: string | null;
  language: Language;
  theme: Theme;
  newProjectOpen: boolean;
  aboutOpen: boolean;
  licenseOpen: boolean;
  languageDialogOpen: boolean;
  themeDialogOpen: boolean;
  exportOpen: boolean;
  leftPanelVisible: boolean;

  /* Масштаб холста */
  zoom: number;

  /* Действия */
  init: () => Promise<void>;
  refreshProject: () => Promise<void>;
  selectBlock: (block: BlockInfo) => Promise<void>;
  loadMore: () => Promise<void>;
  setSearchQuery: (query: string) => Promise<void>;
  openGlyph: (codepoint: number) => Promise<void>;
  closeGlyph: () => void;
  setBrushKind: (kind: BrushKind) => void;
  setBrushSettings: (settings: Partial<BrushSettings>) => void;
  applyPreset: (preset: BrushPreset) => void;
  applyStroke: (stroke: Stroke) => Promise<void>;
  removeStroke: (strokeId: number) => Promise<void>;
  eraseAt: (points: number[][], radius: number) => Promise<void>;
  updateMetrics: (advanceWidth: number, name: string, notes: string) => Promise<void>;
  deleteGlyph: () => Promise<void>;
  undo: () => Promise<void>;
  redo: () => Promise<void>;
  newProject: (familyName: string, styleName: string) => Promise<void>;
  openNewProject: () => void;
  closeNewProject: () => void;
  openAbout: () => void;
  closeAbout: () => void;
  openLicense: () => void;
  closeLicense: () => void;
  openLanguageDialog: () => void;
  closeLanguageDialog: () => void;
  openThemeDialog: () => void;
  closeThemeDialog: () => void;
  openExport: () => void;
  closeExport: () => void;
  toggleLeftPanel: () => void;
  zoomIn: () => void;
  zoomOut: () => void;
  zoomReset: () => void;
  saveProject: (path?: string, forceDialog?: boolean) => Promise<void>;
  openProject: () => Promise<void>;
  exportFont: (format: FontFormat) => Promise<void>;
  decompileFont: () => Promise<void>;
  toggleCursorMode: () => void;
  setSelection: (ids: number[]) => void;
  selectAllStrokes: () => void;
  toggleStrokeSelection: (id: number) => void;
  clearSelection: () => void;
  deleteSelectedStrokes: () => Promise<void>;
  copySelection: () => void;
  cutSelection: () => Promise<void>;
  pasteClipboard: () => Promise<void>;
  moveSelectedStrokes: (dx: number, dy: number) => Promise<void>;
  setStatus: (status: string) => void;
  setError: (error: string | null) => void;
  setLanguage: (language: Language) => void;
  setTheme: (theme: Theme) => void;
}

export const useStore = create<AppState>((set, get) => ({
  project: null,
  meta: null,

  blocks: [],
  selectedBlock: null,
  gridItems: [],
  gridExhausted: false,
  searchQuery: '',
  searchResult: null,
  searching: false,

  currentCodepoint: null,
  glyph: null,
  brushKind: 'pen',
  brushSettings: { ...DEFAULT_BRUSH },
  presets: [],
  nextStrokeId: 1,

  cursorMode: false,
  selectedStrokeIds: [],
  clipboard: [],

  status: translate(loadLanguage(), 'status.ready'),
  error: null,
  language: loadLanguage(),
  theme: loadTheme(),
  newProjectOpen: false,
  aboutOpen: false,
  licenseOpen: false,
  languageDialogOpen: false,
  themeDialogOpen: false,
  exportOpen: false,
  leftPanelVisible: true,

  zoom: 1,

  async init() {
    try {
      const [blocks, presets, project] = await Promise.all([
        ipc.unicodeBlocks(),
        ipc.brushPresets(),
        ipc.projectState(),
      ]);
      set({
        blocks,
        presets,
        project,
        meta: project.meta,
        brushSettings: { ...DEFAULT_BRUSH },
      });

      // По умолчанию открываем первый «интересный» блок.
      const latin = blocks.find((b) => b.name === 'Basic Latin') ?? blocks[0];
      if (latin) {
        await get().selectBlock(latin);
      }
      set({ status: translate(get().language, 'status.blocks', { count: blocks.length }) });
    } catch (error) {
      set({ error: translate(get().language, 'status.error', { message: String(error) }) });
    }
  },

  async refreshProject() {
    const project = await ipc.projectState();
    set({ project, meta: project.meta });
  },

  async selectBlock(block) {
    set({ selectedBlock: block, gridItems: [], gridExhausted: false, searchResult: null });
    await get().loadMore();
  },

  async loadMore() {
    const block = get().selectedBlock;
    if (!block) return;
    const offset = get().gridItems.length;
    const items = await ipc.unicodeRange(block.start, block.end, offset, GRID_PAGE);
    set((state) => ({
      gridItems: [...state.gridItems, ...items],
      gridExhausted: items.length < GRID_PAGE,
    }));
  },

  async setSearchQuery(query) {
    set({ searchQuery: query });
    const trimmed = query.trim();
    if (!trimmed) {
      set({ searchResult: null, searching: false });
      return;
    }
    set({ searching: true });
    try {
      const result = await ipc.unicodeSearch(trimmed, 0, 800);
      set({ searchResult: result, searching: false });
    } catch (error) {
      set({
        error: translate(get().language, 'status.error', { message: String(error) }),
        searching: false,
      });
    }
  },

  async openGlyph(codepoint) {
    const glyph = await ipc.glyphGet(codepoint);
    set({ currentCodepoint: codepoint, glyph, selectedStrokeIds: [] });
  },

  closeGlyph() {
    set({ currentCodepoint: null, glyph: null, selectedStrokeIds: [] });
  },

  setBrushKind(kind) {
    const preset = get().presets.find((p) => p.kind === kind);
    set({ brushKind: kind, brushSettings: preset ? { ...preset.settings } : get().brushSettings });
  },

  setBrushSettings(settings) {
    set((state) => ({ brushSettings: { ...state.brushSettings, ...settings } }));
  },

  applyPreset(preset) {
    set({ brushKind: preset.kind, brushSettings: { ...preset.settings } });
  },

  async applyStroke(stroke) {
    const codepoint = get().currentCodepoint;
    if (codepoint === null) return;
    const glyph = await ipc.glyphUpsertStroke(codepoint, stroke);
    set((state) => ({
      glyph,
      nextStrokeId: Math.max(state.nextStrokeId, stroke.id + 1),
    }));
    await get().refreshProject();
  },

  async removeStroke(strokeId) {
    const codepoint = get().currentCodepoint;
    if (codepoint === null) return;
    const glyph = await ipc.glyphRemoveStroke(codepoint, strokeId);
    set({ glyph });
    await get().refreshProject();
  },

  async eraseAt(points, radius) {
    const codepoint = get().currentCodepoint;
    if (codepoint === null) return;
    const glyph = await ipc.glyphErase(codepoint, points, radius);
    set({ glyph });
    await get().refreshProject();
  },

  async updateMetrics(advanceWidth, name, notes) {
    const codepoint = get().currentCodepoint;
    if (codepoint === null) return;
    const glyph = await ipc.glyphSetMetrics(codepoint, advanceWidth, name, notes);
    set({ glyph });
    await get().refreshProject();
  },

  async deleteGlyph() {
    const codepoint = get().currentCodepoint;
    if (codepoint === null) return;
    await ipc.glyphDelete(codepoint);
    set({ currentCodepoint: null, glyph: null });
    await get().refreshProject();
  },

  async undo() {
    const project = await ipc.projectUndo();
    set({ project, meta: project.meta });
    const codepoint = get().currentCodepoint;
    if (codepoint !== null) {
      set({ glyph: await ipc.glyphGet(codepoint) });
    }
    set({ status: translate(get().language, 'status.undo') });
  },

  async redo() {
    const project = await ipc.projectRedo();
    set({ project, meta: project.meta });
    const codepoint = get().currentCodepoint;
    if (codepoint !== null) {
      set({ glyph: await ipc.glyphGet(codepoint) });
    }
    set({ status: translate(get().language, 'status.redo') });
  },

  toggleCursorMode() {
    set((state) => ({ cursorMode: !state.cursorMode, selectedStrokeIds: [] }));
  },

  setSelection(ids) {
    set({ selectedStrokeIds: ids });
  },

  selectAllStrokes() {
    const glyph = get().glyph;
    if (!glyph) return;
    set({ selectedStrokeIds: glyph.strokes.map((stroke) => stroke.id) });
  },

  toggleStrokeSelection(id) {
    set((state) => ({
      selectedStrokeIds: state.selectedStrokeIds.includes(id)
        ? state.selectedStrokeIds.filter((value) => value !== id)
        : [...state.selectedStrokeIds, id],
    }));
  },

  clearSelection() {
    set({ selectedStrokeIds: [] });
  },

  async deleteSelectedStrokes() {
    const codepoint = get().currentCodepoint;
    const ids = get().selectedStrokeIds;
    if (codepoint === null || ids.length === 0) return;
    let glyph = get().glyph;
    for (const id of ids) {
      glyph = await ipc.glyphRemoveStroke(codepoint, id);
    }
    set({ glyph, selectedStrokeIds: [] });
    await get().refreshProject();
  },

  copySelection() {
    const glyph = get().glyph;
    const ids = get().selectedStrokeIds;
    if (!glyph || ids.length === 0) return;
    const strokes = glyph.strokes
      .filter((stroke) => ids.includes(stroke.id))
      .map((stroke) => ({
        ...stroke,
        points: stroke.points.map((point) => ({ ...point })),
      }));
    set({ clipboard: strokes });
  },

  async cutSelection() {
    get().copySelection();
    await get().deleteSelectedStrokes();
  },

  async pasteClipboard() {
    const codepoint = get().currentCodepoint;
    const clipboard = get().clipboard;
    if (codepoint === null || clipboard.length === 0) return;
    let nextId = get().nextStrokeId;
    const pasted: number[] = [];
    for (const stroke of clipboard) {
      const copy: Stroke = {
        ...stroke,
        id: nextId,
        points: stroke.points.map((point) => ({
          x: point.x + 20,
          y: point.y + 20,
          pressure: point.pressure,
        })),
      };
      nextId += 1;
      const glyph = await ipc.glyphUpsertStroke(codepoint, copy);
      set({ glyph, nextStrokeId: nextId });
      pasted.push(copy.id);
    }
    set({ selectedStrokeIds: pasted });
    await get().refreshProject();
  },

  async moveSelectedStrokes(dx, dy) {
    const codepoint = get().currentCodepoint;
    const ids = get().selectedStrokeIds;
    const glyph = get().glyph;
    if (codepoint === null || !glyph || ids.length === 0) return;
    let updated = glyph;
    for (const stroke of glyph.strokes) {
      if (!ids.includes(stroke.id)) continue;
      const moved: Stroke = {
        ...stroke,
        points: stroke.points.map((point) => ({
          x: point.x + dx,
          y: point.y + dy,
          pressure: point.pressure,
        })),
      };
      updated = await ipc.glyphUpsertStroke(codepoint, moved);
    }
    set({ glyph: updated });
    await get().refreshProject();
  },

  openNewProject() {
    set({ newProjectOpen: true });
  },

  closeNewProject() {
    set({ newProjectOpen: false });
  },

  openAbout() {
    set({ aboutOpen: true });
  },

  closeAbout() {
    set({ aboutOpen: false });
  },

  openLicense() {
    set({ licenseOpen: true });
  },

  closeLicense() {
    set({ licenseOpen: false });
  },

  openLanguageDialog() {
    set({ languageDialogOpen: true });
  },

  closeLanguageDialog() {
    set({ languageDialogOpen: false });
  },

  openThemeDialog() {
    set({ themeDialogOpen: true });
  },

  closeThemeDialog() {
    set({ themeDialogOpen: false });
  },

  toggleLeftPanel() {
    set((state) => ({ leftPanelVisible: !state.leftPanelVisible }));
  },

  zoomIn() {
    set((state) => ({ zoom: Math.min(8, state.zoom * 1.2) }));
  },

  zoomOut() {
    set((state) => ({ zoom: Math.max(0.2, state.zoom / 1.2) }));
  },

  zoomReset() {
    set({ zoom: 1 });
  },

  async newProject(familyName, styleName) {
    try {
      const project = await ipc.projectNew(familyName, styleName);
      set({
        project,
        meta: project.meta,
        currentCodepoint: null,
        glyph: null,
        newProjectOpen: false,
        status: translate(get().language, 'status.created', {
          name: project.meta.familyName,
        }),
      });
    } catch (error) {
      set({ error: translate(get().language, 'status.error', { message: String(error) }) });
    }
  },

  async saveProject(path, forceDialog) {
    try {
      const currentPath = get().project?.path ?? null;
      const target = path ?? (forceDialog || !currentPath ? null : currentPath);
      const project =
        target !== null
          ? await ipc.projectSave(target)
          : await ipc.projectSaveDialog();
      if (!project) return;
      set({
        project,
        meta: project.meta,
        status: translate(get().language, 'status.saved', { path: project.path ?? '' }),
      });
    } catch (error) {
      set({ error: translate(get().language, 'status.error', { message: String(error) }) });
    }
  },

  async openProject() {
    try {
      const project = await ipc.projectOpenDialog();
      if (!project) return;
      set({ project, meta: project.meta, currentCodepoint: null, glyph: null });
      set({ status: translate(get().language, 'status.opened', { path: project.path ?? '' }) });
    } catch (error) {
      set({ error: translate(get().language, 'status.error', { message: String(error) }) });
    }
  },

  openExport() {
    set({ exportOpen: true });
  },

  closeExport() {
    set({ exportOpen: false });
  },

  async exportFont(format: FontFormat) {
    try {
      const report = await ipc.fontExportDialog(format);
      if (!report) return;
      set({
        exportOpen: false,
        status: translate(get().language, 'status.exported', {
          path: report.path,
          glyphs: report.glyphCount,
          bytes: report.bytes,
        }),
      });
    } catch (error) {
      set({ error: translate(get().language, 'status.error', { message: String(error) }) });
    }
  },

  async decompileFont() {
    try {
      const outcome = await ipc.fontDecompileDialog();
      if (!outcome) return;
      set({
        project: outcome.project,
        meta: outcome.project.meta,
        currentCodepoint: null,
        glyph: null,
        status: translate(get().language, 'status.decompiled', {
          family: outcome.familyName,
          style: outcome.styleName,
          glyphs: outcome.glyphCount,
        }),
      });
    } catch (error) {
      set({ error: translate(get().language, 'status.error', { message: String(error) }) });
    }
  },

  setStatus(status) {
    set({ status });
  },
  setError(error) {
    set({ error });
  },

  setLanguage(language) {
    set({ language });
    saveLanguage(language);
    document.documentElement.lang = language;
  },

  setTheme(theme) {
    set({ theme });
    saveTheme(theme);
    document.documentElement.dataset.theme = theme;
  },
}));
