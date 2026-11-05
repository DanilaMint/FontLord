//! Индекс Unicode: блоки, категории, имена и поиск по символам.
//!
//! Индекс строится один раз и кэшируется в статике. Он используется и для
//! построения сетки символов, и для поиска по именам Unicode.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use unicode_categories::UnicodeCategories;

/// Информация о блоке Unicode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockInfo {
    /// Название блока.
    pub name: String,
    /// Первый код символа блока.
    pub start: u32,
    /// Последний код символа блока.
    pub end: u32,
}

impl BlockInfo {
    /// Количество кодов в блоке.
    pub fn len(&self) -> u32 {
        self.end - self.start + 1
    }

    /// Признак пустого блока.
    pub fn is_empty(&self) -> bool {
        self.end < self.start
    }
}

/// Информация о символе Unicode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodepointInfo {
    /// Код символа.
    pub codepoint: u32,
    /// Символ в виде строки (для отображения в сетке).
    pub character: String,
    /// Имя по Unicode.
    pub name: String,
    /// Общая категория (`Lu`, `Nd`, `Po` и т.п.).
    pub category: String,
    /// Название блока.
    pub block: String,
    /// Начало блока.
    pub block_start: u32,
    /// Конец блока.
    pub block_end: u32,
}

/// Результат поиска по Unicode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// Найденные символы.
    pub items: Vec<CodepointInfo>,
    /// Общее количество совпадений.
    pub total: usize,
}

/// Индекс Unicode.
pub struct UnicodeIndex {
    blocks: Vec<BlockInfo>,
    /// Имена символов, склеенные в один буфер.
    names: String,
    /// Имена в нижнем регистре (для поиска без учёта регистра).
    names_lower: String,
    /// Соответствие «код символа → (начало, длина)» в буфере имён.
    entries: Vec<(u32, u32, u32)>,
}

fn build_index() -> UnicodeIndex {
    // 1. Блоки: собираем границы, сканируя диапазон кодов.
    let mut blocks: Vec<BlockInfo> = Vec::new();
    let mut current: Option<BlockInfo> = None;
    for cp in 0..=0x10_FFFFu32 {
        let Some(ch) = char::from_u32(cp) else {
            continue;
        };
        let Some(block) = unicode_blocks::find_unicode_block(ch) else {
            continue;
        };
        match &mut current {
            Some(entry) if entry.start == block.start() => {
                entry.end = cp;
            }
            _ => {
                if let Some(entry) = current.take() {
                    blocks.push(entry);
                }
                current = Some(BlockInfo {
                    name: block.name().to_string(),
                    start: block.start(),
                    end: cp,
                });
            }
        }
    }
    if let Some(entry) = current {
        blocks.push(entry);
    }

    // 2. Имена символов.
    let mut names = String::new();
    let mut names_lower = String::new();
    let mut entries = Vec::new();
    for cp in 0..=0x10_FFFFu32 {
        let Some(ch) = char::from_u32(cp) else {
            continue;
        };
        if (0xD800..=0xDFFF).contains(&cp) {
            continue;
        }
        let Some(name) = unicode_names2::name(ch) else {
            continue;
        };
        let name = name.to_string();
        let start = names.len() as u32;
        names.push_str(&name);
        names_lower.push_str(&name.to_lowercase());
        entries.push((cp, start, name.len() as u32));
    }

    UnicodeIndex {
        blocks,
        names,
        names_lower,
        entries,
    }
}

/// Возвращает глобальный индекс Unicode (строится при первом обращении).
pub fn index() -> &'static UnicodeIndex {
    static INDEX: OnceLock<UnicodeIndex> = OnceLock::new();
    INDEX.get_or_init(build_index)
}

/// Список блоков Unicode.
pub fn blocks() -> &'static [BlockInfo] {
    &index().blocks
}

/// Определяет общую категорию символа.
pub fn category_of(ch: char) -> &'static str {
    if ch.is_letter_uppercase() {
        "Lu"
    } else if ch.is_letter_lowercase() {
        "Ll"
    } else if ch.is_letter_titlecase() {
        "Lt"
    } else if ch.is_letter_other() {
        "Lo"
    } else if ch.is_letter_modifier() {
        "Lm"
    } else if ch.is_mark_nonspacing() {
        "Mn"
    } else if ch.is_mark_spacing_combining() {
        "Mc"
    } else if ch.is_mark_enclosing() {
        "Me"
    } else if ch.is_number_decimal_digit() {
        "Nd"
    } else if ch.is_number_letter() {
        "Nl"
    } else if ch.is_number_other() {
        "No"
    } else if ch.is_punctuation_connector() {
        "Pc"
    } else if ch.is_punctuation_dash() {
        "Pd"
    } else if ch.is_punctuation_close() {
        "Pe"
    } else if ch.is_punctuation_final_quote() {
        "Pf"
    } else if ch.is_punctuation_initial_quote() {
        "Pi"
    } else if ch.is_punctuation_other() {
        "Po"
    } else if ch.is_punctuation_open() {
        "Ps"
    } else if ch.is_symbol_currency() {
        "Sc"
    } else if ch.is_symbol_modifier() {
        "Sk"
    } else if ch.is_symbol_math() {
        "Sm"
    } else if ch.is_symbol_other() {
        "So"
    } else if ch.is_symbol_currency() {
        "Sc"
    } else if ch.is_separator_space() {
        "Zs"
    } else if ch.is_separator_line() {
        "Zl"
    } else if ch.is_separator_paragraph() {
        "Zp"
    } else if ch.is_other_control() {
        "Cc"
    } else if ch.is_other_format() {
        "Cf"
    } else if ch.is_other_private_use() {
        "Co"
    } else {
        "Cn"
    }
}

impl UnicodeIndex {
    /// Имя символа по его коду.
    pub fn name(&self, codepoint: u32) -> Option<&str> {
        let index = self
            .entries
            .binary_search_by_key(&codepoint, |(cp, _, _)| *cp)
            .ok()?;
        let (_, start, len) = self.entries[index];
        Some(&self.names[start as usize..(start + len) as usize])
    }

    /// Список блоков.
    pub fn blocks(&self) -> &[BlockInfo] {
        &self.blocks
    }

    /// Блок, содержащий код символа.
    pub fn block_of(&self, codepoint: u32) -> Option<&BlockInfo> {
        self.blocks
            .iter()
            .find(|b| codepoint >= b.start && codepoint <= b.end)
    }

    /// Полная информация о символе.
    pub fn info(&self, codepoint: u32) -> Option<CodepointInfo> {
        let ch = char::from_u32(codepoint)?;
        let name = self.name(codepoint)?.to_string();
        let block = self.block_of(codepoint);
        Some(CodepointInfo {
            codepoint,
            character: ch.to_string(),
            name,
            category: category_of(ch).to_string(),
            block: block.map(|b| b.name.clone()).unwrap_or_default(),
            block_start: block.map(|b| b.start).unwrap_or(codepoint),
            block_end: block.map(|b| b.end).unwrap_or(codepoint),
        })
    }

    /// Символы диапазона кодов (с пагинацией).
    pub fn range(&self, start: u32, end: u32, offset: usize, limit: usize) -> Vec<CodepointInfo> {
        let mut items = Vec::new();
        let mut skipped = 0usize;
        for (cp, _, _) in &self.entries {
            if *cp < start {
                continue;
            }
            if *cp > end {
                break;
            }
            if skipped < offset {
                skipped += 1;
                continue;
            }
            if let Some(info) = self.info(*cp) {
                items.push(info);
            }
            if items.len() >= limit {
                break;
            }
        }
        items
    }

    /// Поиск по имени символа, коду или самому символу.
    pub fn search(&self, query: &str, offset: usize, limit: usize) -> SearchResult {
        let query = query.trim();
        if query.is_empty() {
            return SearchResult {
                items: Vec::new(),
                total: 0,
            };
        }

        // Поиск по коду вида "U+0041" или "0x41".
        if let Some(cp) = parse_codepoint_query(query) {
            let items: Vec<CodepointInfo> = self.info(cp).into_iter().collect();
            let total = items.len();
            return SearchResult { items, total };
        }

        // Поиск по самому символу.
        let mut chars = query.chars();
        if let (Some(ch), None) = (chars.next(), chars.next()) {
            if let Some(info) = self.info(ch as u32) {
                return SearchResult {
                    items: vec![info],
                    total: 1,
                };
            }
        }

        let needle = query.to_lowercase();
        let mut matches: Vec<(bool, u32)> = Vec::new();
        for (cp, start, len) in &self.entries {
            let slice = &self.names_lower[*start as usize..(*start + *len) as usize];
            if let Some(pos) = slice.find(&needle) {
                matches.push((pos == 0, *cp));
            }
        }

        // Сначала точные префиксы, затем остальные — в порядке кодов.
        matches.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let total = matches.len();
        let items = matches
            .into_iter()
            .skip(offset)
            .take(limit)
            .filter_map(|(_, cp)| self.info(cp))
            .collect();

        SearchResult { items, total }
    }
}

/// Разбирает запрос вида `U+0041`, `u+41`, `0x41`, `0041`.
fn parse_codepoint_query(query: &str) -> Option<u32> {
    let trimmed = query.trim();
    let hex = if let Some(rest) = trimmed
        .strip_prefix("U+")
        .or_else(|| trimmed.strip_prefix("u+"))
        .or_else(|| trimmed.strip_prefix("0x"))
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        rest
    } else if trimmed.len() >= 4 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        trimmed
    } else {
        return None;
    };

    let value = u32::from_str_radix(hex, 16).ok()?;
    char::from_u32(value).map(|_| value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_contains_known_characters() {
        let index = index();
        let info = index.info(0x0041).expect("U+0041");
        assert_eq!(info.character, "A");
        assert_eq!(info.category, "Lu");
        assert!(info.name.contains("LATIN CAPITAL LETTER A"));
    }

    #[test]
    fn blocks_are_sorted_and_unique() {
        let blocks = blocks();
        assert!(!blocks.is_empty());
        for pair in blocks.windows(2) {
            assert!(pair[0].start < pair[1].start);
        }
    }

    #[test]
    fn search_by_name() {
        let result = index().search("snowman", 0, 10);
        assert!(result.total >= 1);
        assert_eq!(result.items[0].codepoint, 0x2603);
    }

    #[test]
    fn search_by_codepoint() {
        let result = index().search("U+0041", 0, 10);
        assert_eq!(result.total, 1);
        assert_eq!(result.items[0].codepoint, 0x0041);
    }

    #[test]
    fn range_returns_block_contents() {
        let items = index().range(0x0041, 0x005A, 0, 100);
        assert_eq!(items.len(), 26);
        assert_eq!(items[0].codepoint, 0x0041);
    }
}
