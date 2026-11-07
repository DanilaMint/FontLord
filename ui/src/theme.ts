/**
 * Темы оформления интерфейса.
 *
 * Тема хранится в localStorage и применяется как атрибут `data-theme`
 * корневого элемента документа: оформление переключается стилями из
 * `styles.css`, а цвета холста берутся из палитры ниже.
 */

/** Поддерживаемые темы. */
export type Theme = 'dark' | 'light';

/** Описание темы для выпадающего списка. */
export interface ThemeOption {
  /** Идентификатор темы. */
  id: Theme;
  /** Название темы. */
  title: string;
}

/** Ключ хранения темы в localStorage. */
const THEME_STORAGE_KEY = 'fontlord.theme';

/** Читает сохранённую тему. */
export function loadTheme(): Theme {
  try {
    const value = window.localStorage.getItem(THEME_STORAGE_KEY);
    if (value === 'dark' || value === 'light') return value;
  } catch {
    /* localStorage может быть недоступен — используем тему по умолчанию */
  }
  return 'dark';
}

/** Сохраняет выбранную тему. */
export function saveTheme(theme: Theme): void {
  try {
    window.localStorage.setItem(THEME_STORAGE_KEY, theme);
  } catch {
    /* localStorage может быть недоступен — просто не сохраняем */
  }
}

/** Цвета холста для каждой темы. */
export interface CanvasPalette {
  /** Фон холста. */
  background: string;
  /** Квадрат em. */
  emSquare: string;
  /** Сетка внутри em. */
  grid: string;
  /** Направляющая. */
  guide: string;
  /** Базовая линия. */
  baseline: string;
  /** Заливка области продвижения. */
  advanceFill: string;
  /** Контур области продвижения. */
  advanceStroke: string;
  /** Заливка контуров глифа. */
  glyphFill: string;
  /** Обводка контуров глифа. */
  glyphStroke: string;
  /** Предпросмотр текущего штриха. */
  preview: string;
  /** Выделение штрихов. */
  selection: string;
}

/** Палитры холста. */
export const CANVAS_PALETTES: Record<Theme, CanvasPalette> = {
  dark: {
    background: '#0f1219',
    emSquare: '#252b38',
    grid: 'rgba(120, 140, 180, 0.08)',
    guide: '#3b4a63',
    baseline: '#5b7fb0',
    advanceFill: 'rgba(120, 190, 255, 0.06)',
    advanceStroke: 'rgba(120, 190, 255, 0.25)',
    glyphFill: '#e9eef7',
    glyphStroke: '#78beff',
    preview: 'rgba(233, 238, 247, 0.85)',
    selection: '#ffb454',
  },
  light: {
    background: '#f6f8fc',
    emSquare: '#dfe5ee',
    grid: 'rgba(40, 60, 100, 0.08)',
    guide: '#b3bfd2',
    baseline: '#4a86c8',
    advanceFill: 'rgba(26, 127, 212, 0.07)',
    advanceStroke: 'rgba(26, 127, 212, 0.30)',
    glyphFill: '#1c2430',
    glyphStroke: '#1a7fd4',
    preview: 'rgba(28, 36, 48, 0.85)',
    selection: '#e07b00',
  },
};
