/**
 * Типизированная обёртка над IPC-командами Tauri.
 */

import { invoke } from '@tauri-apps/api/core';

import type {
  BlockInfo,
  BrushPreset,
  CodepointInfo,
  ExportReport,
  FontMeta,
  Glyph,
  OutlineResult,
  ProjectState,
  SearchResult,
  Stroke,
} from './types';

/** Список блоков Unicode. */
export function unicodeBlocks(): Promise<BlockInfo[]> {
  return invoke<BlockInfo[]>('unicode_blocks');
}

/** Поиск символов по имени, коду или символу. */
export function unicodeSearch(
  query: string,
  offset = 0,
  limit = 500,
): Promise<SearchResult> {
  return invoke<SearchResult>('unicode_search', { query, offset, limit });
}

/** Символы диапазона кодов. */
export function unicodeRange(
  start: number,
  end: number,
  offset = 0,
  limit = 2000,
): Promise<CodepointInfo[]> {
  return invoke<CodepointInfo[]>('unicode_range', { start, end, offset, limit });
}

/** Пресеты кистей. */
export function brushPresets(): Promise<BrushPreset[]> {
  return invoke<BrushPreset[]>('brush_presets');
}

/** Текущее состояние проекта. */
export function projectState(): Promise<ProjectState> {
  return invoke<ProjectState>('project_state_cmd');
}

/** Создание нового проекта. */
export function projectNew(familyName: string, styleName: string): Promise<ProjectState> {
  return invoke<ProjectState>('project_new', { familyName, styleName });
}

/** Открытие проекта через диалог. */
export function projectOpenDialog(): Promise<ProjectState | null> {
  return invoke<ProjectState | null>('project_open_dialog');
}

/** Сохранение проекта через диалог. */
export function projectSaveDialog(): Promise<ProjectState | null> {
  return invoke<ProjectState | null>('project_save_dialog');
}

/** Сохранение проекта в указанный файл. */
export function projectSave(path: string): Promise<ProjectState> {
  return invoke<ProjectState>('project_save', { path: path ?? null });
}

/** Обновление метаданных шрифта. */
export function projectSetMeta(meta: FontMeta): Promise<ProjectState> {
  return invoke<ProjectState>('project_set_meta', { meta });
}

/** Отмена последнего изменения. */
export function projectUndo(): Promise<ProjectState> {
  return invoke<ProjectState>('project_undo');
}

/** Повтор отменённого изменения. */
export function projectRedo(): Promise<ProjectState> {
  return invoke<ProjectState>('project_redo');
}

/** Получение глифа. */
export function glyphGet(codepoint: number): Promise<Glyph | null> {
  return invoke<Glyph | null>('glyph_get', { codepoint });
}

/** Список глифов проекта. */
export function glyphList(): Promise<number[]> {
  return invoke<number[]>('glyph_list');
}

/** Добавление или обновление штриха. */
export function glyphUpsertStroke(codepoint: number, stroke: Stroke): Promise<Glyph> {
  return invoke<Glyph>('glyph_upsert_stroke', { codepoint, stroke });
}

/** Удаление штриха. */
export function glyphRemoveStroke(codepoint: number, strokeId: number): Promise<Glyph> {
  return invoke<Glyph>('glyph_remove_stroke', { codepoint, strokeId });
}

/** Удаление штрихов ластиком. */
export function glyphErase(
  codepoint: number,
  points: number[][],
  radius: number,
): Promise<Glyph> {
  return invoke<Glyph>('glyph_erase', { codepoint, points, radius });
}

/** Изменение метрик глифа. */
export function glyphSetMetrics(
  codepoint: number,
  advanceWidth: number | null,
  name: string | null,
  notes: string | null,
): Promise<Glyph> {
  return invoke<Glyph>('glyph_set_metrics', {
    codepoint,
    advanceWidth,
    name,
    notes,
  });
}

/** Удаление глифа. */
export function glyphDelete(codepoint: number): Promise<void> {
  return invoke<void>('glyph_delete', { codepoint });
}

/** Контуры глифа. */
export function glyphOutline(codepoint: number): Promise<OutlineResult | null> {
  return invoke<OutlineResult | null>('glyph_outline', { codepoint });
}

/** Экспорт шрифта через диалог. */
export function fontExportDialog(): Promise<ExportReport | null> {
  return invoke<ExportReport | null>('font_export_dialog');
}

/** Байты шрифта для предпросмотра в интерфейсе. */
export function fontPreviewBytes(): Promise<number[]> {
  return invoke<number[]>('font_preview_ttf');
}
