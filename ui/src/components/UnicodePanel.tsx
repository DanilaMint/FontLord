/**
 * Панель Unicode: дерево блоков, поиск и сетка символов.
 */

import { useEffect, useMemo, useRef, useState } from 'react';

import { useStore } from '../store';
import type { CodepointInfo } from '../types';

export function UnicodePanel() {  const blocks = useStore((state) => state.blocks);
  const selectedBlock = useStore((state) => state.selectedBlock);
  const gridItems = useStore((state) => state.gridItems);
  const gridExhausted = useStore((state) => state.gridExhausted);
  const searchQuery = useStore((state) => state.searchQuery);
  const searchResult = useStore((state) => state.searchResult);
  const searching = useStore((state) => state.searching);
  const selectBlock = useStore((state) => state.selectBlock);
  const loadMore = useStore((state) => state.loadMore);
  const setSearchQuery = useStore((state) => state.setSearchQuery);
  const openGlyph = useStore((state) => state.openGlyph);
  const currentCodepoint = useStore((state) => state.currentCodepoint);
  const project = useStore((state) => state.project);

  const [blockFilter, setBlockFilter] = useState('');
  const [query, setQuery] = useState(searchQuery);
  const debounceRef = useRef<number | null>(null);

  // Поиск с задержкой, чтобы не дёргать IPC на каждый символ.
  useEffect(() => {
    if (debounceRef.current !== null) window.clearTimeout(debounceRef.current);
    debounceRef.current = window.setTimeout(() => {
      void setSearchQuery(query);
    }, 180);
    return () => {
      if (debounceRef.current !== null) window.clearTimeout(debounceRef.current);
    };
  }, [query, setSearchQuery]);

  const filteredBlocks = useMemo(() => {
    const needle = blockFilter.trim().toLowerCase();
    if (!needle) return blocks;
    return blocks.filter((block) => block.name.toLowerCase().includes(needle));
  }, [blockFilter, blocks]);

  const items: CodepointInfo[] = searchResult ? searchResult.items : gridItems;
  const total = searchResult ? searchResult.total : gridItems.length;

  return (
    <div className="panel unicode-panel">
      <div className="panel-header">
        <h2>Символы</h2>
        <span className="badge">{searchResult ? `найдено ${total}` : `${total}`}</span>
      </div>

      <div className="search-row">
        <input
          className="input"
          type="search"
          placeholder="Поиск: имя, U+0041, символ…"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
        />
      </div>

      {!searchResult && (
        <div className="block-filter">
          <input
            className="input small"
            type="search"
            placeholder="Фильтр блоков"
            value={blockFilter}
            onChange={(event) => setBlockFilter(event.target.value)}
          />
        </div>
      )}

      {!searchResult && (
        <div className="block-list">
          {filteredBlocks.map((block) => (
            <button
              type="button"
              key={block.name}
              className={`block-item${
                selectedBlock?.name === block.name ? ' selected' : ''
              }`}
              onClick={() => void selectBlock(block)}
              title={`U+${block.start.toString(16).toUpperCase()}–U+${block.end
                .toString(16)
                .toUpperCase()}`}
            >
              <span className="block-name">{block.name}</span>
              <span className="block-range">
                {block.start.toString(16).toUpperCase()}–{block.end.toString(16).toUpperCase()}
              </span>
            </button>
          ))}
        </div>
      )}

      {searching && <div className="hint">Поиск…</div>}

      <div className="glyph-grid">
        {items.map((item) => {
          const drawn = project?.glyphs.includes(item.codepoint) ?? false;
          return (
            <button
              type="button"
              key={item.codepoint}
              className={`glyph-cell${
                currentCodepoint === item.codepoint ? ' current' : ''
              }${drawn ? ' drawn' : ''}`}
              onClick={() => void openGlyph(item.codepoint)}
              title={`${item.name}\nU+${item.codepoint
                .toString(16)
                .toUpperCase()
                .padStart(4, '0')} · ${item.category} · ${item.block}`}
            >
              <span className="glyph-char">{displayChar(item)}</span>
              <span className="glyph-code">
                {item.codepoint.toString(16).toUpperCase().padStart(4, '0')}
              </span>
            </button>
          );
        })}
      </div>

      {!searchResult && !gridExhausted && (
        <button type="button" className="load-more" onClick={() => void loadMore()}>
          Показать ещё
        </button>
      )}
    </div>
  );
}

/**
 * Символ для ячейки сетки.
 *
 * Символы области частного использования и незанятые коды не имеют
 * начертания, поэтому для них показывается техническая пометка.
 */
function displayChar(item: CodepointInfo): string {
  if (item.category === 'Co') return 'PUA';
  if (item.category === 'Cn') return '—';
  return item.character;
}
