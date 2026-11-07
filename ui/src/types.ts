/**
 * Типы данных, зеркалящие модели Rust-ядра (`fontlord-core`).
 *
 * Сериализация Rust использует `camelCase`, поэтому имена полей совпадают
 * с интерфейсами ниже один в один.
 */

/** Тип кисти. */
export type BrushKind = 'pen' | 'marker' | 'broadNib' | 'soft' | 'eraser';

/** Параметры кисти. */
export interface BrushSettings {
  /** Номинальная ширина мазка в шрифтовых единицах. */
  width: number;
  /** Угол пера в радианах. */
  angle: number;
  /** Минимальная доля ширины при нулевом нажиме. */
  minWidthRatio: number;
  /** Экспонента отклика на нажим. */
  pressureGamma: number;
  /** Доля длины штриха для сужения в начале. */
  taperIn: number;
  /** Доля длины штриха для сужения в конце. */
  taperOut: number;
  /** Жёсткость края. */
  hardness: number;
}

/** Пресет кисти. */
export interface BrushPreset {
  id: string;
  kind: BrushKind;
  settings: BrushSettings;
  name: string;
  description: string;
}

/** Точка штриха. */
export interface StrokePoint {
  x: number;
  y: number;
  pressure: number;
}

/** Штрих — нарисованная пользователем линия. */
export interface Stroke {
  id: number;
  kind: BrushKind;
  settings: BrushSettings;
  points: StrokePoint[];
  closed: boolean;
}

/** Точка контура. */
export interface ContourPoint {
  x: number;
  y: number;
  onCurve: boolean;
}

/** Контур глифа. */
export interface Contour {
  points: ContourPoint[];
}

/** Якорь глифа. */
export interface Anchor {
  name: string;
  x: number;
  y: number;
}

/** Ссылка на другой глиф. */
export interface GlyphComponent {
  codepoint: number;
  dx: number;
  dy: number;
  scale: number;
}

/** Глиф проекта. */
export interface Glyph {
  name: string;
  codepoint: number | null;
  advanceWidth: number;
  strokes: Stroke[];
  contours: Contour[];
  anchors: Anchor[];
  components: GlyphComponent[];
  notes: string;
}

/** Метаданные шрифта. */
export interface FontMeta {
  familyName: string;
  styleName: string;
  version: string;
  unitsPerEm: number;
  ascender: number;
  descender: number;
  lineGap: number;
  xHeight: number;
  capHeight: number;
  designer: string;
  copyright: string;
  license: string;
}

/** Состояние проекта для интерфейса. */
export interface ProjectState {
  meta: FontMeta;
  glyphCount: number;
  glyphs: number[];
  path: string | null;
  dirty: boolean;
  canUndo: boolean;
  canRedo: boolean;
}

/** Информация о блоке Unicode. */
export interface BlockInfo {
  name: string;
  start: number;
  end: number;
}

/** Информация о символе Unicode. */
export interface CodepointInfo {
  codepoint: number;
  character: string;
  name: string;
  category: string;
  block: string;
  blockStart: number;
  blockEnd: number;
}

/** Результат поиска. */
export interface SearchResult {
  items: CodepointInfo[];
  total: number;
}

/** Результат построения контура. */
export interface OutlineResult {
  contours: Contour[];
  bbox: {
    xMin: number;
    yMin: number;
    xMax: number;
    yMax: number;
  };
}

/** Формат экспортируемого шрифта. */
export type FontFormat = 'ttf' | 'otf' | 'woff' | 'woff2';

/** Отчёт об экспорте. */
export interface ExportReport {
  path: string;
  bytes: number;
  glyphCount: number;
  unitsPerEm: number;
  warnings: string[];
}

/** Результат декомпиляции шрифта. */
export interface DecompileOutcome {
  project: ProjectState;
  familyName: string;
  styleName: string;
  glyphCount: number;
}
