/**
 * Словари интерфейса.
 *
 * Модуль не зависит от состояния приложения: его можно использовать и в
 * сторе, и в компонентах. Хук `useT` живёт в `i18n.ts`.
 */

/** Поддерживаемые языки интерфейса. */
export type Language = 'ru' | 'en';

/** Описание языка для выпадающего списка. */
export interface LanguageOption {
  /** Идентификатор языка. */
  id: Language;
  /** Название языка (на своём языке). */
  title: string;
}

/** Список языков интерфейса. */
export const LANGUAGES: LanguageOption[] = [
  { id: 'ru', title: 'Русский' },
  { id: 'en', title: 'English' },
];

/** Значения, подставляемые в шаблон перевода. */
export type TranslationVars = Record<string, string | number>;

/** Словари: ключ → шаблон. */
const DICTIONARIES: Record<Language, Record<string, string>> = {
  ru: {
    /* --- Меню --- */
    'menu.file': 'Файл',
    'menu.edit': 'Правка',
    'menu.view': 'Вид',
    'menu.brushes': 'Кисти',
    'menu.settings': 'Настройки',
    'menu.help': 'Справка',
    'menu.file.new': 'Новый проект…',
    'menu.file.open': 'Открыть проект…',
    'menu.file.save': 'Сохранить',
    'menu.file.saveAs': 'Сохранить как…',
    'menu.file.decompile': 'Декомпилировать шрифт…',
    'menu.file.export': 'Экспортировать шрифт…',
    'menu.file.quit': 'Выход',
    'menu.edit.undo': 'Отменить',
    'menu.edit.redo': 'Повторить',
    'menu.edit.deleteGlyph': 'Удалить глиф',
    'menu.edit.closeGlyph': 'Закрыть глиф',
    'menu.view.zoomIn': 'Увеличить',
    'menu.view.zoomOut': 'Уменьшить',
    'menu.view.zoomReset': 'Сбросить масштаб',
    'menu.view.panel': 'Панель символов',
    'menu.brushes.pen': 'Перо',
    'menu.brushes.marker': 'Маркер',
    'menu.brushes.broadNib': 'Каллиграфия',
    'menu.brushes.soft': 'Мягкая кисть',
    'menu.brushes.eraser': 'Ластик',
    'menu.settings.language': 'Язык…',
    'menu.settings.theme': 'Тема…',
    'menu.help.about': 'О программе',
    'menu.help.license': 'Лицензия',

    /* --- Панель инструментов --- */
    'toolbar.subtitle': 'редактор шрифтов',
    'toolbar.undo': 'Отменить',
    'toolbar.redo': 'Повторить',
    'toolbar.create': 'Создать',
    'toolbar.open': 'Открыть',
    'toolbar.save': 'Сохранить',
    'toolbar.export': 'Экспорт',

    /* --- Рабочая область --- */
    'workspace.selectGlyph': 'Выберите символ для редактирования',
    'workspace.close': 'Закрыть',
    'status.glyphs': 'Глифов: {glyphs} · Единиц на em: {upem}',

    /* --- Панель символов --- */
    'unicode.title': 'Символы',
    'unicode.found': 'найдено {count}',
    'unicode.count': '{count}',
    'unicode.searchPlaceholder': 'Поиск: имя, U+0041, символ…',
    'unicode.filterPlaceholder': 'Фильтр блоков',
    'unicode.searching': 'Поиск…',
    'unicode.loadMore': 'Показать ещё',

    /* --- Кисти --- */
    'brushes.title': 'Кисти',
    'brush.width': 'Толщина',
    'brush.angle': 'Угол пера',
    'brush.minWidth': 'Мин. толщина',
    'brush.pressure': 'Отклик на нажим',
    'brush.taperIn': 'Сужение в начале',
    'brush.taperOut': 'Сужение в конце',
    'brush.units': 'ед.',
    'preset.pen-fine': 'Тонкое перо',
    'preset.pen-bold': 'Жирное перо',
    'preset.marker': 'Маркер',
    'preset.calligraphy-45': 'Каллиграфия 45°',
    'preset.calligraphy-30': 'Каллиграфия 30°',
    'preset.soft-ink': 'Тушь',
    'preset.eraser': 'Ластик',

    /* --- Инспектор глифа --- */
    'glyph.title': 'Глиф',
    'glyph.empty': 'Глиф пуст. Нарисуйте первый штрих на холсте — глиф будет создан автоматически.',
    'glyph.choose': 'Выберите символ в сетке слева, чтобы начать рисование.',
    'glyph.name': 'Имя',
    'glyph.width': 'Ширина',
    'glyph.notes': 'Заметки',
    'glyph.strokes': 'Штрихи ({count})',
    'glyph.delete': 'Удалить глиф',
    'glyph.noStrokes': 'Штрихов пока нет. Рисуйте кистью на холсте.',
    'glyph.strokeInfo': '{kind} · {count} точек · {width} ед.',

    /* --- Параметры шрифта --- */
    'font.title': 'Шрифт',
    'font.family': 'Семейство',
    'font.style': 'Начертание',
    'font.version': 'Версия',
    'font.upem': 'Единиц на em',
    'font.ascender': 'Ascender',
    'font.descender': 'Descender',
    'font.xHeight': 'x-height',
    'font.capHeight': 'Cap-height',
    'font.saved': 'Параметры шрифта обновлены',

    /* --- Предпросмотр --- */
    'preview.title': 'Предпросмотр шрифта',
    'preview.hint':
      'Предпросмотр использует реальную сборку шрифта: контуры глифов, метрики и кернинг.',

    /* --- Холст --- */
    'canvas.ring': 'Кольцо',
    'canvas.shiftHint': 'Shift — прямая',
    'canvas.zoomReset': '100%',

    /* --- Окна --- */
    'about.title': 'Fontlord',
    'about.subtitle': 'Редактор шрифтов с векторными кистями',
    'about.text':
      'Символы рисуются обычной мышью, а специальные кисти превращают рисунок в настоящие векторные контуры TrueType. Проект сохраняется в файл .fontlord, готовый шрифт экспортируется в форматах .ttf, .otf, .woff и .woff2.',
    'about.close': 'Закрыть',
    'about.hotkey.project': 'новый проект, открыть, сохранить',
    'about.hotkey.export': 'экспортировать шрифт',
    'about.hotkey.history': 'отменить, повторить',
    'about.hotkey.brushes': 'перо, маркер, каллиграфия, мягкая кисть, ластик',
    'about.hotkey.zoom': 'масштаб холста',
    'about.hotkey.glyph': 'удалить глиф, закрыть глиф',
    'about.hotkey.cursor': 'режим курсора: выделение и перемещение штрихов',

    'license.title': 'Лицензия Fontlord',
    'license.summary':
      'Код программы можно свободно читать, изменять и распространять. Программу как таковую продавать нельзя, зато декомпиляция и исследование кода прямо разрешены. Все шрифты, созданные в Fontlord, принадлежат вам: их можно свободно использовать и продавать.',
    'license.close': 'Закрыть',

    'export.title': 'Экспорт шрифта',
    'export.hint': 'Выберите формат собираемого файла.',
    'export.ttf': 'Классический формат шрифтов, поддерживается везде.',
    'export.otf': 'Контуры CFF вместо квадратичных кривых TrueType.',
    'export.woff': 'Веб-формат: те же таблицы, сжатие zlib.',
    'export.woff2': 'Веб-формат со сжатием Brotli — самый компактный.',
    'export.cancel': 'Отмена',

    'new.title': 'Новый проект',
    'new.hint': 'Имя семейства и начертания попадут в таблицу `name` собранного шрифта.',
    'new.family': 'Семейство',
    'new.style': 'Начертание',
    'new.cancel': 'Отмена',
    'new.create': 'Создать',

    /* --- Настройки --- */
    'settings.language.title': 'Язык интерфейса',
    'settings.language.hint': 'Язык сохраняется между запусками приложения.',
    'settings.theme.title': 'Тема оформления',
    'settings.theme.hint': 'Тема сохраняется между запусками приложения.',
    'settings.close': 'Закрыть',
    'theme.dark': 'Тёмная',
    'theme.light': 'Светлая',

    /* --- Статус --- */
    'status.ready': 'Готово',
    'status.blocks': 'Загружено блоков: {count}',
    'status.undo': 'Отмена',
    'status.redo': 'Повтор',
    'status.created': 'Создан проект «{name}»',
    'status.saved': 'Сохранено: {path}',
    'status.opened': 'Открыт: {path}',
    'status.exported': 'Шрифт сохранён: {path} ({glyphs} глифов, {bytes} Б)',
    'status.decompiled':
      'Декомпилирован шрифт «{family} {style}» — глифов: {glyphs}',
    'status.error': 'Ошибка: {message}',
  },
  en: {
    /* --- Menu --- */
    'menu.file': 'File',
    'menu.edit': 'Edit',
    'menu.view': 'View',
    'menu.brushes': 'Brushes',
    'menu.settings': 'Settings',
    'menu.help': 'Help',
    'menu.file.new': 'New project…',
    'menu.file.open': 'Open project…',
    'menu.file.save': 'Save',
    'menu.file.saveAs': 'Save as…',
    'menu.file.decompile': 'Decompile font…',
    'menu.file.export': 'Export font…',
    'menu.file.quit': 'Quit',
    'menu.edit.undo': 'Undo',
    'menu.edit.redo': 'Redo',
    'menu.edit.deleteGlyph': 'Delete glyph',
    'menu.edit.closeGlyph': 'Close glyph',
    'menu.view.zoomIn': 'Zoom in',
    'menu.view.zoomOut': 'Zoom out',
    'menu.view.zoomReset': 'Reset zoom',
    'menu.view.panel': 'Character panel',
    'menu.brushes.pen': 'Pen',
    'menu.brushes.marker': 'Marker',
    'menu.brushes.broadNib': 'Calligraphy',
    'menu.brushes.soft': 'Soft brush',
    'menu.brushes.eraser': 'Eraser',
    'menu.settings.language': 'Language…',
    'menu.settings.theme': 'Theme…',
    'menu.help.about': 'About',
    'menu.help.license': 'License',

    /* --- Toolbar --- */
    'toolbar.subtitle': 'font editor',
    'toolbar.undo': 'Undo',
    'toolbar.redo': 'Redo',
    'toolbar.create': 'New',
    'toolbar.open': 'Open',
    'toolbar.save': 'Save',
    'toolbar.export': 'Export',

    /* --- Workspace --- */
    'workspace.selectGlyph': 'Select a character to edit',
    'workspace.close': 'Close',
    'status.glyphs': 'Glyphs: {glyphs} · Units per em: {upem}',

    /* --- Character panel --- */
    'unicode.title': 'Characters',
    'unicode.found': 'found {count}',
    'unicode.count': '{count}',
    'unicode.searchPlaceholder': 'Search: name, U+0041, character…',
    'unicode.filterPlaceholder': 'Filter blocks',
    'unicode.searching': 'Searching…',
    'unicode.loadMore': 'Show more',

    /* --- Brushes --- */
    'brushes.title': 'Brushes',
    'brush.width': 'Width',
    'brush.angle': 'Nib angle',
    'brush.minWidth': 'Min. width',
    'brush.pressure': 'Pressure response',
    'brush.taperIn': 'Taper in',
    'brush.taperOut': 'Taper out',
    'brush.units': 'u.',
    'preset.pen-fine': 'Fine pen',
    'preset.pen-bold': 'Bold pen',
    'preset.marker': 'Marker',
    'preset.calligraphy-45': 'Calligraphy 45°',
    'preset.calligraphy-30': 'Calligraphy 30°',
    'preset.soft-ink': 'Ink',
    'preset.eraser': 'Eraser',

    /* --- Glyph inspector --- */
    'glyph.title': 'Glyph',
    'glyph.empty': 'The glyph is empty. Draw the first stroke — the glyph will be created automatically.',
    'glyph.choose': 'Pick a character in the grid on the left to start drawing.',
    'glyph.name': 'Name',
    'glyph.width': 'Width',
    'glyph.notes': 'Notes',
    'glyph.strokes': 'Strokes ({count})',
    'glyph.delete': 'Delete glyph',
    'glyph.noStrokes': 'No strokes yet. Draw with a brush on the canvas.',
    'glyph.strokeInfo': '{kind} · {count} points · {width} u.',

    /* --- Font settings --- */
    'font.title': 'Font',
    'font.family': 'Family',
    'font.style': 'Style',
    'font.version': 'Version',
    'font.upem': 'Units per em',
    'font.ascender': 'Ascender',
    'font.descender': 'Descender',
    'font.xHeight': 'x-height',
    'font.capHeight': 'Cap-height',
    'font.saved': 'Font settings updated',

    /* --- Preview --- */
    'preview.title': 'Font preview',
    'preview.hint':
      'The preview uses a real font build: glyph outlines, metrics and kerning.',

    /* --- Canvas --- */
    'canvas.ring': 'Ring',
    'canvas.shiftHint': 'Shift — straight line',
    'canvas.zoomReset': '100%',

    /* --- Dialogs --- */
    'about.title': 'Fontlord',
    'about.subtitle': 'Font editor with vector brushes',
    'about.text':
      'Characters are drawn with an ordinary mouse, and special brushes turn the drawing into real TrueType vector outlines. Projects are stored in .fontlord files; finished fonts are exported as .ttf, .otf, .woff and .woff2.',
    'about.close': 'Close',
    'about.hotkey.project': 'new project, open, save',
    'about.hotkey.export': 'export font',
    'about.hotkey.history': 'undo, redo',
    'about.hotkey.brushes': 'pen, marker, calligraphy, soft brush, eraser',
    'about.hotkey.zoom': 'canvas zoom',
    'about.hotkey.glyph': 'delete glyph, close glyph',
    'about.hotkey.cursor': 'cursor mode: select and move strokes',

    'license.title': 'Fontlord license',
    'license.summary':
      'The source code may be read, modified and redistributed freely. The program itself may not be sold, while decompiling and studying the code is expressly permitted. Every font you create with Fontlord belongs to you: use and sell it without restrictions.',
    'license.close': 'Close',

    'export.title': 'Export font',
    'export.hint': 'Choose the format of the file to build.',
    'export.ttf': 'Classic font format, supported everywhere.',
    'export.otf': 'CFF outlines instead of TrueType quadratic curves.',
    'export.woff': 'Web format: the same tables, zlib compression.',
    'export.woff2': 'Web format with Brotli compression — the most compact.',
    'export.cancel': 'Cancel',

    'new.title': 'New project',
    'new.hint': 'The family and style names end up in the `name` table of the built font.',
    'new.family': 'Family',
    'new.style': 'Style',
    'new.cancel': 'Cancel',
    'new.create': 'Create',

    /* --- Settings --- */
    'settings.language.title': 'Interface language',
    'settings.language.hint': 'The language is remembered between application runs.',
    'settings.theme.title': 'Color theme',
    'settings.theme.hint': 'The theme is remembered between application runs.',
    'settings.close': 'Close',
    'theme.dark': 'Dark',
    'theme.light': 'Light',

    /* --- Status --- */
    'status.ready': 'Ready',
    'status.blocks': 'Loaded blocks: {count}',
    'status.undo': 'Undo',
    'status.redo': 'Redo',
    'status.created': 'Created project “{name}”',
    'status.saved': 'Saved: {path}',
    'status.opened': 'Opened: {path}',
    'status.exported': 'Font saved: {path} ({glyphs} glyphs, {bytes} B)',
    'status.decompiled': 'Decompiled font “{family} {style}” — glyphs: {glyphs}',
    'status.error': 'Error: {message}',
  },
};

/** Ключ хранения выбранного языка в localStorage. */
const LANGUAGE_STORAGE_KEY = 'fontlord.language';

/** Читает сохранённый язык интерфейса. */
export function loadLanguage(): Language {
  try {
    const value = window.localStorage.getItem(LANGUAGE_STORAGE_KEY);
    if (value === 'ru' || value === 'en') return value;
  } catch {
    /* localStorage может быть недоступен — используем язык по умолчанию */
  }
  return 'ru';
}

/** Сохраняет выбранный язык интерфейса. */
export function saveLanguage(language: Language): void {
  try {
    window.localStorage.setItem(LANGUAGE_STORAGE_KEY, language);
  } catch {
    /* localStorage может быть недоступен — просто не сохраняем */
  }
}

/**
 * Переводит ключ на выбранный язык.
 *
 * Если ключа нет в выбранном языке, используется русский вариант, а если его
 * нет и там — сам ключ (чтобы ошибка была заметна).
 */
export function translate(
  language: Language,
  key: string,
  vars?: TranslationVars,
): string {
  const table = DICTIONARIES[language] ?? DICTIONARIES.ru;
  const template = table[key] ?? DICTIONARIES.ru[key] ?? key;
  if (!vars) return template;
  return template.replace(/\{(\w+)\}/g, (_, name: string) => {
    const value = vars[name];
    return value === undefined ? `{${name}}` : String(value);
  });
}
