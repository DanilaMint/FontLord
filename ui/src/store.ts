/**
 * Глобальное состояние приложения (zustand).
 *
 * Хранилище отвечает за загрузку данных (Unicode-индекс, проект, глифы) и
 * за вызовы IPC. Компоненты только читают состояние и вызывают действия.
 */

import { create } from 'zustand';

import * as ipc from './ipc';
import type {
  BlockInfo,
  BrushKind,
  BrushPreset,
  BrushSettings,
  CodepointInfo,
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
  saveProject: (path?: string) => Promise<void>;
  openProject: () => Promise<void>;
  exportFont: () => Promise<void>;
  setStatus: (status: string) => void;
  setError: (error: string | null) => void;
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

  status: 'Готово',
  error: null,

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
      set({ status: `Загружено блоков: ${blocks.length}` });
    } catch (error) {
      set({ error: String(error) });
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
      set({ error: String(error), searching: false });
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
    set({ status: 'Отмена' });
  },

  async redo() {
    const project = await ipc.projectRedo();
    set({ project, meta: project.meta });
    const codepoint = get().currentCodepoint;
    if (codepoint !== null) {
      set({ glyph: await ipc.glyphGet(codepoint) });
    }
    set({ status: 'Повтор' });
  },

  async newProject(familyName, styleName) {
    try {
      const project = await ipc.projectNew(familyName, styleName);
      set({
        project,
        meta: project.meta,
        currentCodepoint: null,
        glyph: null,
        status: `Создан проект «${project.meta.familyName}»`,
      });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  async saveProject(path) {
    try {
      const project = path ? await ipc.projectSave(path) : await ipc.projectSaveDialog();
      if (!project) return;
      set({ project, meta: project.meta, status: `Сохранено: ${project.path ?? ''}` });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  async openProject() {
    try {
      const project = await ipc.projectOpenDialog();
      if (!project) return;
      set({ project, meta: project.meta, currentCodepoint: null, glyph: null });
      set({ status: `Открыт: ${project.path ?? ''}` });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  async exportFont() {
    try {
      const report = await ipc.fontExportDialog();
      if (!report) return;
      set({
        status: `Шрифт сохранён: ${report.path} (${report.glyphCount} глифов, ${report.bytes} Б)`,
      });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  setStatus(status) {
    set({ status });
  },

  setError(error) {
    set({ error });
  },
}));
