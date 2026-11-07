/**
 * Глобальное состояние приложения (zustand).
 *
 * Хранилище отвечает за загрузку данных (Unicode-индекс, проект, глифы) и
 * за вызовы IPC. Компоненты только читают состояние и вызывают действия.
 */

import { create } from 'zustand';

import { loadLanguage, saveLanguage, translate, type Language } from './dict';
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

  /* Служебное */
  status: string;
  error: string | null;
  language: Language;
  newProjectOpen: boolean;
  aboutOpen: boolean;
  licenseOpen: boolean;
  languageDialogOpen: boolean;
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
  setStatus: (status: string) => void;
  setError: (error: string | null) => void;
  setLanguage: (language: Language) => void;
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

  status: translate(loadLanguage(), 'status.ready'),
  error: null,
  language: loadLanguage(),
  newProjectOpen: false,
  aboutOpen: false,
  licenseOpen: false,
  languageDialogOpen: false,
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
    set({ currentCodepoint: codepoint, glyph });
  },

  closeGlyph() {
    set({ currentCodepoint: null, glyph: null });
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
}));
