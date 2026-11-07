/**
 * Верхнее меню приложения.
 *
 * Поведение как в стандартных приложениях: меню открывается по клику, при
 * открытом меню наведение переключаетmenu, закрытие — по клику вне меню,
 * клавише Escape или выбору пункта. Поддерживается навигация стрелками.
 */

import { useEffect, useRef, useState } from 'react';

import { useStore } from '../store';

/** Пункт меню. */
export interface MenuItem {
  /** Текст пункта. */
  label: string;
  /** Сочетание клавиш, показываемое справа. */
  shortcut?: string;
  /** Действие. */
  action?: () => void;
  /** Пункт недоступен. */
  disabled?: boolean;
  /** Пункт отмечен галочкой. */
  checked?: boolean;
  /** Разделитель между пунктами. */
  separator?: boolean;
}

/** Меню верхнего уровня. */
export interface Menu {
  /** Заголовок меню. */
  label: string;
  /** Пункты меню. */
  items: MenuItem[];
}

/** Собирает меню из текущего состояния приложения. */
function useMenus(): Menu[] {
  const project = useStore((state) => state.project);
  const glyph = useStore((state) => state.glyph);
  const brushKind = useStore((state) => state.brushKind);

  const openNewProject = useStore((state) => state.openNewProject);
  const openProject = useStore((state) => state.openProject);
  const saveProject = useStore((state) => state.saveProject);
  const openExport = useStore((state) => state.openExport);
  const undo = useStore((state) => state.undo);
  const redo = useStore((state) => state.redo);
  const deleteGlyph = useStore((state) => state.deleteGlyph);
  const closeGlyph = useStore((state) => state.closeGlyph);
  const setBrushKind = useStore((state) => state.setBrushKind);
  const zoomIn = useStore((state) => state.zoomIn);
  const zoomOut = useStore((state) => state.zoomOut);
  const zoomReset = useStore((state) => state.zoomReset);
  const toggleLeftPanel = useStore((state) => state.toggleLeftPanel);
  const leftPanelVisible = useStore((state) => state.leftPanelVisible);
  const openAbout = useStore((state) => state.openAbout);
  const openLicense = useStore((state) => state.openLicense);

  return [
    {
      label: 'Файл',
      items: [
        { label: 'Новый проект…', shortcut: 'Ctrl+N', action: openNewProject },
        { label: 'Открыть проект…', shortcut: 'Ctrl+O', action: () => void openProject() },
        { separator: true, label: '' },
        { label: 'Сохранить', shortcut: 'Ctrl+S', action: () => void saveProject() },
        {
          label: 'Сохранить как…',
          shortcut: 'Ctrl+Shift+S',
          action: () => void saveProject(undefined, true),
        },
        { separator: true, label: '' },
        { label: 'Экспортировать шрифт…', shortcut: 'Ctrl+E', action: openExport },
        { separator: true, label: '' },
        { label: 'Выход', shortcut: 'Ctrl+Q', action: () => void window.close() },
      ],
    },
    {
      label: 'Правка',
      items: [
        {
          label: 'Отменить',
          shortcut: 'Ctrl+Z',
          disabled: !project?.canUndo,
          action: () => void undo(),
        },
        {
          label: 'Повторить',
          shortcut: 'Ctrl+Y',
          disabled: !project?.canRedo,
          action: () => void redo(),
        },
        { separator: true, label: '' },
        {
          label: 'Удалить глиф',
          shortcut: 'Del',
          disabled: !glyph,
          action: () => void deleteGlyph(),
        },
        {
          label: 'Закрыть глиф',
          shortcut: 'Esc',
          disabled: !glyph,
          action: closeGlyph,
        },
      ],
    },
    {
      label: 'Вид',
      items: [
        { label: 'Увеличить', shortcut: 'Ctrl++', action: zoomIn },
        { label: 'Уменьшить', shortcut: 'Ctrl+-', action: zoomOut },
        { label: 'Сбросить масштаб', shortcut: 'Ctrl+0', action: zoomReset },
        { separator: true, label: '' },
        {
          label: 'Панель символов',
          shortcut: 'Ctrl+B',
          checked: leftPanelVisible,
          action: toggleLeftPanel,
        },
      ],
    },
    {
      label: 'Кисти',
      items: [
        {
          label: 'Перо',
          shortcut: 'B',
          action: () => setBrushKind('pen'),
          disabled: brushKind === 'pen',
        },
        {
          label: 'Маркер',
          shortcut: 'M',
          action: () => setBrushKind('marker'),
          disabled: brushKind === 'marker',
        },
        {
          label: 'Каллиграфия',
          shortcut: 'N',
          action: () => setBrushKind('broadNib'),
          disabled: brushKind === 'broadNib',
        },
        {
          label: 'Мягкая кисть',
          shortcut: 'S',
          action: () => setBrushKind('soft'),
          disabled: brushKind === 'soft',
        },
        {
          label: 'Ластик',
          shortcut: 'E',
          action: () => setBrushKind('eraser'),
          disabled: brushKind === 'eraser',
        },
      ],
    },
    {
      label: 'Справка',
      items: [
        { label: 'О программе', shortcut: 'F1', action: openAbout },
        { separator: true, label: '' },
        { label: 'Лицензия', action: openLicense },
      ],
    },
  ];
}

export function MenuBar() {
  const menus = useMenus();
  const [openIndex, setOpenIndex] = useState<number | null>(null);
  const [activeIndex, setActiveIndex] = useState(0);
  const rootRef = useRef<HTMLDivElement>(null);

  // Закрытие меню по клику вне него.
  useEffect(() => {
    if (openIndex === null) return;
    const onPointerDown = (event: PointerEvent) => {
      if (rootRef.current && !rootRef.current.contains(event.target as Node)) {
        setOpenIndex(null);
      }
    };
    window.addEventListener('pointerdown', onPointerDown);
    return () => window.removeEventListener('pointerdown', onPointerDown);
  }, [openIndex]);

  // Навигация стрелками внутри открытого меню.
  useEffect(() => {
    if (openIndex === null) return;
    const onKeyDown = (event: KeyboardEvent) => {
      const items = menus[openIndex].items.filter((item) => !item.separator);
      if (event.key === 'ArrowDown') {
        event.preventDefault();
        setActiveIndex((value) => (value + 1) % items.length);
      } else if (event.key === 'ArrowUp') {
        event.preventDefault();
        setActiveIndex((value) => (value - 1 + items.length) % items.length);
      } else if (event.key === 'Enter') {
        event.preventDefault();
        const item = items[activeIndex];
        setOpenIndex(null);
        item.action?.();
      } else if (event.key === 'Escape') {
        event.preventDefault();
        setOpenIndex(null);
      }
    };
    window.addEventListener('keydown', onKeyDown, true);
    return () => window.removeEventListener('keydown', onKeyDown, true);
  }, [activeIndex, menus, openIndex]);

  const openMenu = (index: number) => {
    setActiveIndex(0);
    setOpenIndex((current) => (current === index ? null : index));
  };

  return (
    <div className="menubar" ref={rootRef}>
      {menus.map((menu, index) => (
        <div
          key={menu.label}
          className="menubar-item"
          onPointerEnter={() => {
            if (openIndex !== null) openMenu(index);
          }}
        >
          <button
            type="button"
            className={`menubar-button${openIndex === index ? ' open' : ''}`}
            onClick={() => openMenu(index)}
            onPointerDown={(event) => event.preventDefault()}
          >
            {menu.label}
          </button>

          {openIndex === index && (
            <div className="menubar-dropdown">
              {menu.items.map((item, itemIndex) =>
                item.separator ? (
                  <div key={`sep-${itemIndex}`} className="menubar-separator" />
                ) : (
                  <button
                    key={item.label}
                    type="button"
                    className="menubar-entry"
                    disabled={item.disabled}
                    onPointerEnter={() => {
                      const items = menu.items.filter((entry) => !entry.separator);
                      setActiveIndex(items.indexOf(item));
                    }}
                    onClick={() => {
                      setOpenIndex(null);
                      item.action?.();
                    }}
                  >
                    <span className="menubar-check">{item.checked ? '✓' : ''}</span>
                    <span className="menubar-label">{item.label}</span>
                    {item.shortcut && <kbd>{item.shortcut}</kbd>}
                  </button>
                ),
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
