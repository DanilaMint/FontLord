//! Построение таблицы `CFF ` (Compact Font Format).
//!
//! Таблица `CFF ` хранит контуры шрифта в виде charstring'ов Type 2 —
//! формата, который используется в шрифтах OpenType с расширением `.otf`.
//! Квадратичные кривые TrueType переводятся в кубические, как того требует
//! формат Type 2.
//!
//! Структура файла CFF:
//!
//! ```text
//! заголовок | Name INDEX | Top DICT INDEX | String INDEX | Global Subr INDEX
//!           | charset | CharStrings INDEX | Private DICT
//! ```

use fontlord_core::model::FontProject;

use crate::glyf::GlyphPoint;
use crate::writer::Writer;

/// Глиф, подготовленный для сборки charstring'ов.
pub(crate) struct CffGlyph {
    /// Имя глифа.
    pub name: String,
    /// Ширина продвижения.
    pub advance_width: u16,
    /// Нормализованные квадратичные контуры.
    pub contours: Vec<Vec<GlyphPoint>>,
}

/// Количество стандартных строк CFF (идентификаторы SID 0..=390).
const STANDARD_STRINGS: u16 = 391;

/// Собирает таблицу `CFF ` для шрифта.
pub(crate) fn build_cff_table(project: &FontProject, glyphs: &[CffGlyph]) -> Vec<u8> {
    // 1. Пользовательские строки: имена глифов и сведения о шрифте.
    let mut strings: Vec<String> = Vec::new();
    let ps_name = format!(
        "{}-{}",
        sanitize_ps_name(&project.meta.family_name),
        sanitize_ps_name(&project.meta.style_name)
    );
    let full_name_sid = add_string(
        &mut strings,
        &format!("{} {}", project.meta.family_name, project.meta.style_name),
    );
    let family_sid = add_string(&mut strings, &project.meta.family_name);
    let weight_sid = add_string(&mut strings, &project.meta.style_name);
    let notice_sid = add_string(&mut strings, &project.meta.copyright);
    let version_sid = add_string(&mut strings, &project.meta.version);

    // 2. Идентификаторы глифов: `.notdef` имеет SID 0, остальные — строки
    //    пользовательского набора.
    let mut sids: Vec<u16> = Vec::with_capacity(glyphs.len());
    for (index, glyph) in glyphs.iter().enumerate() {
        if index == 0 {
            sids.push(0);
            continue;
        }
        let name = sanitize_ps_name(&glyph.name);
        sids.push(add_string(&mut strings, &name));
    }

    // 3. Charstring'и глифов.
    let charstrings: Vec<Vec<u8>> = glyphs.iter().map(build_charstring).collect();

    // 4. Таблица charset: сопоставление индексов глифов и имён.
    let mut charset = Writer::new();
    charset.u8(0); // формат 0
    for sid in sids.iter().skip(1) {
        charset.u16(*sid);
    }

    // 5. Приватный словарь: ширины глифов по умолчанию.
    let mut private = Writer::new();
    private.number(0); // defaultWidthX
    private.u8(20);
    private.number(0); // nominalWidthX
    private.u8(21);

    // 6. Границы шрифта.
    let mut bbox = [0i32; 4];
    for glyph in glyphs {
        for contour in &glyph.contours {
            for point in contour {
                bbox[0] = bbox[0].min(point.x as i32);
                bbox[1] = bbox[1].min(point.y as i32);
                bbox[2] = bbox[2].max(point.x as i32);
                bbox[3] = bbox[3].max(point.y as i32);
            }
        }
    }

    // 7. Индексы. Размер смещения в них общий — он задаётся в заголовке
    //    файла CFF.
    let name_items = vec![ps_name.as_bytes().to_vec()];
    let string_items: Vec<Vec<u8>> = strings
        .iter()
        .map(|value| value.as_bytes().to_vec())
        .collect();
    let charstring_items = charstrings;

    let off_size = [
        index_data_size(&name_items),
        index_data_size(&string_items),
        index_data_size(&charstring_items),
    ]
    .into_iter()
    .max()
    .unwrap_or(1)
    .max(1);
    let off_size = off_size_bytes(off_size + 1);

    // Top DICT: смещения кодируются пятибайтовыми числами, поэтому длина
    // словаря не зависит от значений смещений.
    let top_dict = build_top_dict(
        0,
        0,
        0,
        private.bytes.len(),
        &bbox,
        version_sid,
        notice_sid,
        full_name_sid,
        family_sid,
        weight_sid,
    );
    let top_dict_items = vec![top_dict];

    let name_index = build_index_with(&name_items, off_size);
    let top_dict_index = build_index_with(&top_dict_items, off_size);
    let string_index = build_index_with(&string_items, off_size);
    let global_subr_index = build_index_with(&[], off_size);
    let charstrings_index = build_index_with(&charstring_items, off_size);

    // 8. Раскладка файла.
    let header_size = 4usize;
    let name_pos = header_size;
    let top_dict_pos = name_pos + name_index.len();
    let string_pos = top_dict_pos + top_dict_index.len();
    let subr_pos = string_pos + string_index.len();
    let charset_pos = subr_pos + global_subr_index.len();
    let charstrings_pos = charset_pos + charset.bytes.len();
    let private_pos = charstrings_pos + charstrings_index.len();

    let top_dict = build_top_dict(
        charset_pos,
        charstrings_pos,
        private_pos,
        private.bytes.len(),
        &bbox,
        version_sid,
        notice_sid,
        full_name_sid,
        family_sid,
        weight_sid,
    );
    debug_assert_eq!(top_dict.len(), top_dict_items[0].len());
    let top_dict_index = build_index_with(&[top_dict], off_size);

    let mut result = Writer::new();
    result.u8(1); // major
    result.u8(0); // minor
    result.u8(4); // hdrSize
    result.u8(off_size as u8);
    result.bytes(&name_index);
    result.bytes(&top_dict_index);
    result.bytes(&string_index);
    result.bytes(&global_subr_index);
    result.bytes(&charset.bytes);
    result.bytes(&charstrings_index);
    result.bytes(&private.bytes);
    result.bytes
}

/// Суммарный размер данных INDEX'а.
fn index_data_size(items: &[Vec<u8>]) -> usize {
    items.iter().map(|item| item.len()).sum()
}

/// Собирает INDEX с заданным размером смещения.
fn build_index_with(items: &[Vec<u8>], off_size: usize) -> Vec<u8> {
    let mut writer = Writer::new();
    if items.is_empty() {
        writer.u16(0);
        return writer.bytes;
    }
    writer.u16(items.len() as u16);
    writer.u8(off_size as u8);

    let mut offset = 1usize;
    write_offset(&mut writer, offset, off_size);
    for item in items {
        offset += item.len();
        write_offset(&mut writer, offset, off_size);
    }
    for item in items {
        writer.bytes(item);
    }
    writer.bytes
}

/// Число байт, необходимое для хранения смещения.
fn off_size_bytes(value: usize) -> usize {
    if value < 0x100 {
        1
    } else if value < 0x1_0000 {
        2
    } else if value < 0x100_0000 {
        3
    } else {
        4
    }
}

/// Записывает смещение в INDEX'е.
fn write_offset(writer: &mut Writer, offset: usize, off_size: usize) {
    let bytes = (offset as u64).to_be_bytes();
    for byte in bytes.iter().skip(8 - off_size) {
        writer.u8(*byte);
    }
}

/// Добавляет строку в пользовательский набор и возвращает её SID.
fn add_string(strings: &mut Vec<String>, value: &str) -> u16 {
    if let Some(index) = strings.iter().position(|item| item == value) {
        return STANDARD_STRINGS + index as u16;
    }
    strings.push(value.to_string());
    STANDARD_STRINGS + (strings.len() - 1) as u16
}

/// Приводит имя к виду PostScript-имени.
fn sanitize_ps_name(name: &str) -> String {
    let mut result = String::new();
    for character in name.chars() {
        if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
            result.push(character);
        } else if character == ' ' || character == '.' {
            result.push('-');
        }
    }
    if result.is_empty() {
        result.push_str("glyph");
    }
    if !result.starts_with(|c: char| c.is_ascii_alphabetic()) {
        result.insert(0, 'g');
    }
    result.truncate(63);
    result
}

/// Собирает Top DICT шрифта.
#[allow(clippy::too_many_arguments)]
fn build_top_dict(
    charset_offset: usize,
    charstrings_offset: usize,
    private_offset: usize,
    private_size: usize,
    bbox: &[i32; 4],
    version_sid: u16,
    notice_sid: u16,
    full_name_sid: u16,
    family_sid: u16,
    weight_sid: u16,
) -> Vec<u8> {
    let mut dict = Writer::new();

    dict.number(version_sid as i32);
    dict.u8(0); // version
    dict.number(notice_sid as i32);
    dict.u8(1); // Notice
    dict.number(full_name_sid as i32);
    dict.u8(2); // FullName
    dict.number(family_sid as i32);
    dict.u8(3); // FamilyName
    dict.number(weight_sid as i32);
    dict.u8(4); // Weight
    for value in bbox {
        dict.number(*value);
    }
    dict.u8(5); // FontBBox
    dict.number(0);
    dict.u8(12);
    dict.u8(2); // ItalicAngle
    dict.number(-100);
    dict.u8(12);
    dict.u8(3); // UnderlinePosition
    dict.number(50);
    dict.u8(12);
    dict.u8(4); // UnderlineThickness
    dict.number(0);
    dict.u8(12);
    dict.u8(1); // isFixedPitch
    dict.number(0);
    dict.u8(12);
    dict.u8(5); // PaintType
    dict.number(0);
    dict.u8(12);
    dict.u8(6); // CharstringType

    dict.offset(charset_offset as u32);
    dict.u8(15); // charset
    dict.offset(charstrings_offset as u32);
    dict.u8(17); // CharStrings
    dict.number(private_size as i32);
    dict.offset(private_offset as u32);
    dict.u8(18); // Private

    dict.bytes
}

/// Сегмент контура в кубическом представлении.
enum Segment {
    /// Прямая.
    Line { x: i32, y: i32 },
    /// Кубическая кривая Безье.
    Curve {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x: i32,
        y: i32,
    },
}

/// Переводит квадратичный контур в последовательность кубических сегментов.
///
/// Возвращает сегменты и начальную точку контура.
fn to_cubic_segments(points: &[GlyphPoint]) -> Option<(Vec<Segment>, (i32, i32))> {
    if points.len() < 2 {
        return None;
    }
    let count = points.len();
    let start_index = points.iter().position(|point| point.on_curve).unwrap_or(0);
    let start = points[start_index];
    let start_point = (start.x as i32, start.y as i32);

    let mut segments = Vec::new();
    let mut current = start_point;
    let mut index = (start_index + 1) % count;
    let mut guard = 0usize;

    while index != start_index && guard <= count * 2 {
        guard += 1;
        let point = points[index];
        if point.on_curve {
            let target = (point.x as i32, point.y as i32);
            if target != current {
                segments.push(Segment::Line {
                    x: target.0,
                    y: target.1,
                });
            }
            current = target;
            index = (index + 1) % count;
            continue;
        }

        let control = (point.x as i32, point.y as i32);
        let next_index = (index + 1) % count;
        let next = points[next_index];
        let end = if next.on_curve {
            (next.x as i32, next.y as i32)
        } else {
            // Подразумеваемая точка на кривой — середина между
            // управляющей и следующей точкой.
            (
                (control.0 + next.x as i32) / 2,
                (control.1 + next.y as i32) / 2,
            )
        };

        // Квадратичная кривая превращается в кубическую:
        // C1 = Q0 + 2/3 (Q1 - Q0), C2 = Q2 + 2/3 (Q1 - Q2).
        let c1 = (
            current.0 + 2 * (control.0 - current.0) / 3,
            current.1 + 2 * (control.1 - current.1) / 3,
        );
        let c2 = (
            end.0 + 2 * (control.0 - end.0) / 3,
            end.1 + 2 * (control.1 - end.1) / 3,
        );
        segments.push(Segment::Curve {
            x1: c1.0,
            y1: c1.1,
            x2: c2.0,
            y2: c2.1,
            x: end.0,
            y: end.1,
        });
        current = end;
        index = (next_index + 1) % count;
    }

    // Замыкаем контур.
    if current != start_point {
        segments.push(Segment::Line {
            x: start_point.0,
            y: start_point.1,
        });
    }

    Some((segments, start_point))
}

/// Собирает charstring глифа в формате Type 2.
fn build_charstring(glyph: &CffGlyph) -> Vec<u8> {
    let mut writer = Writer::new();
    // Ширина передаётся первым операндом первого оператора. Ширина по
    // умолчанию (defaultWidthX) равна нулю, поэтому при нулевой ширине
    // операнд не нужен.
    let mut pending_width = if glyph.advance_width != 0 {
        Some(glyph.advance_width as i32)
    } else {
        None
    };

    // Текущая точка charstring'а: операторы `rmoveto`, `rlineto` и
    // `rrcurveto` задают смещения относительно неё.
    let mut current = (0i32, 0i32);

    for contour in &glyph.contours {
        let Some((segments, start)) = to_cubic_segments(contour) else {
            continue;
        };
        if segments.is_empty() {
            continue;
        }

        if let Some(width) = pending_width.take() {
            write_charstring_number(&mut writer, width);
        }
        write_charstring_number(&mut writer, start.0 - current.0);
        write_charstring_number(&mut writer, start.1 - current.1);
        writer.u8(21); // rmoveto
        current = start;
        for segment in &segments {
            match segment {
                Segment::Line { x, y } => {
                    write_charstring_number(&mut writer, x - current.0);
                    write_charstring_number(&mut writer, y - current.1);
                    writer.u8(5); // rlineto
                    current = (*x, *y);
                }
                Segment::Curve {
                    x1,
                    y1,
                    x2,
                    y2,
                    x,
                    y,
                } => {
                    write_charstring_number(&mut writer, x1 - current.0);
                    write_charstring_number(&mut writer, y1 - current.1);
                    write_charstring_number(&mut writer, x2 - x1);
                    write_charstring_number(&mut writer, y2 - y1);
                    write_charstring_number(&mut writer, x - x2);
                    write_charstring_number(&mut writer, y - y2);
                    writer.u8(8); // rrcurveto
                    current = (*x, *y);
                }
            }
        }
    }

    if let Some(width) = pending_width.take() {
        write_charstring_number(&mut writer, width);
    }
    writer.u8(14); // endchar
    writer.bytes
}

/// Записывает число в charstring (формат Type 2).
fn write_charstring_number(writer: &mut Writer, value: i32) {
    if (-107..=107).contains(&value) {
        writer.u8((value + 139) as u8);
    } else if (-1131..=-108).contains(&value) {
        let value = -value - 108;
        writer.u8(((value >> 8) + 251) as u8);
        writer.u8((value & 0xFF) as u8);
    } else if (108..=1131).contains(&value) {
        let value = value - 108;
        writer.u8(((value >> 8) + 247) as u8);
        writer.u8((value & 0xFF) as u8);
    } else if (-32768..=32767).contains(&value) {
        writer.u8(28);
        writer.u16(value as u16);
    } else {
        // 16.16 фиксированное число.
        writer.u8(255);
        writer.u32(((value as i64) << 16) as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rectangle() -> Vec<GlyphPoint> {
        vec![
            GlyphPoint {
                x: 0,
                y: 0,
                on_curve: true,
            },
            GlyphPoint {
                x: 100,
                y: 0,
                on_curve: true,
            },
            GlyphPoint {
                x: 100,
                y: 200,
                on_curve: true,
            },
            GlyphPoint {
                x: 0,
                y: 200,
                on_curve: true,
            },
        ]
    }

    #[test]
    fn converts_quadratic_contour_to_cubic() {
        let points = vec![
            GlyphPoint {
                x: 0,
                y: 0,
                on_curve: true,
            },
            GlyphPoint {
                x: 50,
                y: 100,
                on_curve: false,
            },
            GlyphPoint {
                x: 100,
                y: 0,
                on_curve: true,
            },
        ];
        let (segments, start) = to_cubic_segments(&points).unwrap();
        assert_eq!(start, (0, 0));
        // Кривая и замыкающий отрезок.
        assert_eq!(segments.len(), 2);
        match &segments[0] {
            Segment::Curve { x, y, .. } => {
                assert_eq!((*x, *y), (100, 0));
            }
            Segment::Line { .. } => panic!("ожидалась кривая"),
        }
        match &segments[1] {
            Segment::Line { x, y } => {
                assert_eq!((*x, *y), (0, 0));
            }
            Segment::Curve { .. } => panic!("ожидался отрезок"),
        }
    }

    #[test]
    fn charstring_ends_with_endchar() {
        let glyph = CffGlyph {
            name: "A".to_string(),
            advance_width: 600,
            contours: vec![rectangle()],
        };
        let program = build_charstring(&glyph);
        assert_eq!(program.last(), Some(&14));
    }

    #[test]
    fn sanitizes_names() {
        assert_eq!(sanitize_ps_name("A B"), "A-B");
        assert_eq!(sanitize_ps_name("1"), "g1");
        assert_eq!(sanitize_ps_name(""), "glyph");
    }
}
