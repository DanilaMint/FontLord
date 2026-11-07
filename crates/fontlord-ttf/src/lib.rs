//! # Fontlord TTF
//!
//! Сборщики шрифтов: TrueType (`glyf`), OpenType с контурами CFF (`.otf`),
//! а также веб-форматы WOFF и WOFF2.
//!
//! Принимают проект [`fontlord_core`] и возвращают готовый файл `.ttf`,
//! `.otf`, `.woff` или `.woff2`, который можно установить в систему, загрузить
//! в браузер или открыть в любом редакторе шрифтов.
//!
//! Поддерживаемые таблицы TrueType: `head`, `hhea`, `maxp`, `hmtx`, `cmap`
//! (форматы 4 и 12), `glyf`, `loca`, `name`, `OS/2`, `post`, `kern`.

use std::collections::BTreeMap;

use fontlord_core::model::{Contour, FontProject, Glyph};
use fontlord_core::outline::{OutlineSettings, build_glyph_outline};

pub mod cff;
pub mod decompile;
pub mod glyf;
pub mod woff;
pub mod woff2;
pub mod writer;

pub use decompile::{DecompileReport, decompile_font};
pub use woff::build_woff;
pub use woff2::build_woff2;

use glyf::{GlyphPoint, SimpleGlyph, encode_simple_glyph, normalize_contour};
use writer::{Writer, checksum};

/// Таблица шрифта: четырёхсимвольный тег и содержимое.
pub(crate) type Table = (&'static [u8; 4], Vec<u8>);

/// Версия sfnt для шрифтов TrueType.
pub(crate) const SFNT_TRUETYPE: u32 = 0x0001_0000;

/// Версия sfnt для шрифтов OpenType с контурами CFF.
pub(crate) const SFNT_CFF: u32 = 0x4F54_544F;

/// Ошибка сборки шрифта.
#[derive(Debug, thiserror::Error)]
pub enum FontError {
    /// В проекте нет ни одного глифа.
    #[error("в проекте нет глифов — нечего экспортировать")]
    EmptyProject,

    /// Некорректные метрики проекта.
    #[error("некорректные метрики: {0}")]
    InvalidMetrics(String),

    /// Ошибка ввода-вывода.
    #[error("ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),

    /// Не удалось разобрать файл шрифта.
    #[error("не удалось прочитать шрифт: {0}")]
    Parse(String),
}

/// Прежнее имя ошибки сборки шрифта.
pub type TtfError = FontError;

/// Поддерживаемые форматы экспорта.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFormat {
    /// TrueType: контуры в таблице `glyf`. Расширение `.ttf`.
    Ttf,
    /// OpenType: контуры CFF в таблице `CFF `. Расширение `.otf`.
    Otf,
    /// Web Open Font Format 1.0. Расширение `.woff`.
    Woff,
    /// Web Open Font Format 2.0. Расширение `.woff2`.
    Woff2,
}

impl FontFormat {
    /// Расширение файла для формата.
    pub fn extension(self) -> &'static str {
        match self {
            FontFormat::Ttf => "ttf",
            FontFormat::Otf => "otf",
            FontFormat::Woff => "woff",
            FontFormat::Woff2 => "woff2",
        }
    }

    /// Определяет формат по расширению файла.
    pub fn from_extension(extension: &str) -> Option<Self> {
        let extension = extension
            .trim()
            .trim_start_matches('.')
            .to_ascii_lowercase();
        match extension.as_str() {
            "ttf" => Some(FontFormat::Ttf),
            "otf" => Some(FontFormat::Otf),
            "woff" => Some(FontFormat::Woff),
            "woff2" => Some(FontFormat::Woff2),
            _ => None,
        }
    }

    /// Человекочитаемое название формата.
    pub fn label(self) -> &'static str {
        match self {
            FontFormat::Ttf => "TrueType (.ttf)",
            FontFormat::Otf => "OpenType CFF (.otf)",
            FontFormat::Woff => "Web Open Font Format (.woff)",
            FontFormat::Woff2 => "Web Open Font Format 2 (.woff2)",
        }
    }

    /// Все форматы в порядке отображения в интерфейсе.
    pub fn all() -> [FontFormat; 4] {
        [
            FontFormat::Ttf,
            FontFormat::Otf,
            FontFormat::Woff,
            FontFormat::Woff2,
        ]
    }
}

/// Подготовленный глиф шрифта.
pub(crate) struct PreparedGlyph {
    /// Контуры в квадратичном представлении (TrueType).
    pub contours: Vec<Contour>,
    /// Глиф в виде простого глифа таблицы `glyf`.
    pub simple: SimpleGlyph,
    /// Ширина продвижения.
    pub advance_width: u16,
    /// Левое боковое поле.
    pub lsb: i16,
    /// Имя глифа.
    pub name: String,
}

/// Готовый глиф шрифта.
struct BuiltGlyph {
    /// Данные таблицы `glyf`.
    data: Vec<u8>,
    /// Ширина продвижения.
    advance_width: u16,
    /// Левое боковое поле.
    lsb: i16,
}

/// Собирает шрифт в выбранном формате.
pub fn build_font(project: &FontProject, format: FontFormat) -> Result<Vec<u8>, FontError> {
    match format {
        FontFormat::Ttf => build_truetype(project),
        FontFormat::Otf => build_opentype(project),
        FontFormat::Woff => woff::build_woff(project),
        FontFormat::Woff2 => woff2::build_woff2(project),
    }
}

/// Собирает шрифт TrueType из проекта.
pub fn build_truetype(project: &FontProject) -> Result<Vec<u8>, FontError> {
    let tables = truetype_tables(project)?;
    Ok(assemble(SFNT_TRUETYPE, tables))
}

/// Собирает шрифт OpenType с контурами CFF из проекта.
pub fn build_opentype(project: &FontProject) -> Result<Vec<u8>, FontError> {
    let tables = opentype_tables(project)?;
    Ok(assemble(SFNT_CFF, tables))
}

/// Готовит глифы проекта: контуры, метрики и карту кодов в индексы глифов.
pub(crate) fn prepare_glyphs(
    project: &FontProject,
) -> Result<(Vec<PreparedGlyph>, BTreeMap<u32, u16>), FontError> {
    let upem = project.meta.units_per_em;
    if !(16..=16384).contains(&upem) {
        return Err(FontError::InvalidMetrics(format!(
            "единицы на em должны быть в диапазоне 16..=16384, получено {upem}"
        )));
    }

    // Список глифов: .notdef + глифы проекта по возрастанию кода.
    let mut entries: Vec<(u32, &Glyph)> = project.glyphs.iter().map(|(cp, g)| (*cp, g)).collect();
    entries.sort_by_key(|(cp, _)| *cp);

    let mut prepared: Vec<PreparedGlyph> = Vec::with_capacity(entries.len() + 1);
    prepared.push(PreparedGlyph {
        contours: Vec::new(),
        simple: SimpleGlyph::default(),
        advance_width: upem / 2,
        lsb: 0,
        name: ".notdef".to_string(),
    });

    let mut codepoint_to_gid: BTreeMap<u32, u16> = BTreeMap::new();

    let settings = OutlineSettings::default();
    for (gid, (codepoint, glyph)) in (1u16..).zip(entries.iter()) {
        let contours = collect_contours(glyph, &settings);
        let simple = to_simple_glyph(&contours);
        let lsb = if simple.bbox.is_empty() {
            0
        } else {
            simple.bbox.x_min
        };
        prepared.push(PreparedGlyph {
            contours,
            simple,
            advance_width: glyph.advance_width,
            lsb,
            name: if glyph.name.trim().is_empty() {
                fontlord_core::unicode::postscript_name(*codepoint)
            } else {
                glyph.name.clone()
            },
        });
        codepoint_to_gid.insert(*codepoint, gid);
    }

    Ok((prepared, codepoint_to_gid))
}

/// Собирает контуры глифа, включая раскрытие компонентов.
fn collect_contours(glyph: &Glyph, settings: &OutlineSettings) -> Vec<Contour> {
    // Компоненты (основа лигатур и диакритики) пока раскрываются в контуры
    // сдвигом: это гарантирует корректное отображение без составных глифов.
    build_glyph_outline(glyph, settings)
}

/// Преобразует контуры в структуру простого глифа.
fn to_simple_glyph(contours: &[Contour]) -> SimpleGlyph {
    let mut result = SimpleGlyph::default();
    for contour in contours {
        if contour.points.len() < 2 {
            continue;
        }
        let normalized = normalize_contour(&contour.points);
        if normalized.len() < 2 {
            continue;
        }
        result.contours.push(normalized);
    }
    result.bbox = glyf::bbox_of(&result.contours);
    result
}

/// Общие метрики шрифта.
struct FontMetrics {
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
    advance_max: u16,
    min_lsb: i16,
    min_rsb: i16,
    max_extent: i16,
    max_points: u16,
    max_contours: u16,
}

/// Считает метрики шрифта по подготовленным глифам.
fn compute_metrics(prepared: &[PreparedGlyph]) -> FontMetrics {
    let mut x_min = i16::MAX;
    let mut y_min = i16::MAX;
    let mut x_max = i16::MIN;
    let mut y_max = i16::MIN;
    let mut advance_max = 0u16;
    let mut min_lsb = i16::MAX;
    let mut min_rsb = i16::MAX;
    let mut max_extent = i16::MIN;
    let mut max_points = 0u16;
    let mut max_contours = 0u16;

    for glyph in prepared {
        advance_max = advance_max.max(glyph.advance_width);
        if glyph.simple.contours.is_empty() {
            continue;
        }
        let bbox = glyph.simple.bbox;
        x_min = x_min.min(bbox.x_min);
        y_min = y_min.min(bbox.y_min);
        x_max = x_max.max(bbox.x_max);
        y_max = y_max.max(bbox.y_max);
        min_lsb = min_lsb.min(glyph.lsb);
        min_rsb = min_rsb.min(glyph.advance_width as i16 - glyph.lsb);
        max_extent = max_extent.max(bbox.x_max);

        let points: u16 = glyph
            .simple
            .contours
            .iter()
            .map(|contour| contour.len() as u16)
            .sum();
        max_points = max_points.max(points);
        max_contours = max_contours.max(glyph.simple.contours.len() as u16);
    }

    if x_min > x_max {
        x_min = 0;
        y_min = 0;
        x_max = 0;
        y_max = 0;
    }
    if min_lsb > x_max {
        min_lsb = 0;
    }
    if min_rsb > x_max {
        min_rsb = 0;
    }
    if max_extent < x_min {
        max_extent = x_max;
    }

    FontMetrics {
        x_min,
        y_min,
        x_max,
        y_max,
        advance_max,
        min_lsb,
        min_rsb,
        max_extent,
        max_points,
        max_contours,
    }
}

/// Кодирует подготовленные глифы в данные таблицы `glyf`.
fn build_glyf_data(prepared: &[PreparedGlyph]) -> Vec<BuiltGlyph> {
    prepared
        .iter()
        .map(|glyph| BuiltGlyph {
            data: encode_simple_glyph(&glyph.simple),
            advance_width: glyph.advance_width,
            lsb: glyph.lsb,
        })
        .collect()
}

/// Таблицы шрифта TrueType.
pub(crate) fn truetype_tables(project: &FontProject) -> Result<Vec<Table>, FontError> {
    let (prepared, codepoint_to_gid) = prepare_glyphs(project)?;
    let metrics = compute_metrics(&prepared);
    let built = build_glyf_data(&prepared);

    let glyf_table = build_glyf_table(&built);
    let loca_table = build_loca_table(&built);
    let cmap_table = build_cmap_table(&codepoint_to_gid);
    let hmtx_table = build_hmtx_table(&built);
    let kern_table = build_kern_table(project, &codepoint_to_gid);
    let name_table = build_name_table(project);
    let os2_table = build_os2_table(project, &codepoint_to_gid, &prepared);
    let post_table = build_post_table(project);
    let head_table = build_head_table(
        project,
        (metrics.x_min, metrics.y_min, metrics.x_max, metrics.y_max),
        project.meta.units_per_em,
        true,
    );
    let hhea_table = build_hhea_table(
        project,
        metrics.advance_max,
        metrics.min_lsb,
        metrics.min_rsb,
        metrics.max_extent,
        prepared.len() as u16,
    );
    let maxp_table = build_maxp_table(
        prepared.len() as u16,
        metrics.max_points,
        metrics.max_contours,
    );

    Ok(vec![
        (b"OS/2", os2_table),
        (b"cmap", cmap_table),
        (b"glyf", glyf_table),
        (b"head", head_table),
        (b"hhea", hhea_table),
        (b"hmtx", hmtx_table),
        (b"kern", kern_table),
        (b"loca", loca_table),
        (b"maxp", maxp_table),
        (b"name", name_table),
        (b"post", post_table),
    ])
}

/// Таблицы шрифта OpenType с контурами CFF.
fn opentype_tables(project: &FontProject) -> Result<Vec<Table>, FontError> {
    let (prepared, codepoint_to_gid) = prepare_glyphs(project)?;
    let metrics = compute_metrics(&prepared);

    let glyphs: Vec<cff::CffGlyph> = prepared
        .iter()
        .map(|glyph| cff::CffGlyph {
            name: glyph.name.clone(),
            advance_width: glyph.advance_width,
            contours: glyph
                .contours
                .iter()
                .map(|contour| normalize_contour(&contour.points))
                .filter(|contour| contour.len() >= 2)
                .collect(),
        })
        .collect();

    let cff_table = cff::build_cff_table(project, &glyphs);
    let cmap_table = build_cmap_table(&codepoint_to_gid);
    let hmtx_table = build_hmtx_table(
        &prepared
            .iter()
            .map(|glyph| BuiltGlyph {
                data: Vec::new(),
                advance_width: glyph.advance_width,
                lsb: glyph.lsb,
            })
            .collect::<Vec<_>>(),
    );
    let kern_table = build_kern_table(project, &codepoint_to_gid);
    let name_table = build_name_table(project);
    let os2_table = build_os2_table(project, &codepoint_to_gid, &prepared);
    let post_table = build_post_table(project);
    let head_table = build_head_table(
        project,
        (metrics.x_min, metrics.y_min, metrics.x_max, metrics.y_max),
        project.meta.units_per_em,
        false,
    );
    let hhea_table = build_hhea_table(
        project,
        metrics.advance_max,
        metrics.min_lsb,
        metrics.min_rsb,
        metrics.max_extent,
        prepared.len() as u16,
    );
    // Для шрифтов с контурами CFF таблица `maxp` имеет версию 0.5 и содержит
    // только количество глифов.
    let maxp_table = build_maxp_cff_table(prepared.len() as u16);

    Ok(vec![
        (b"CFF ", cff_table),
        (b"OS/2", os2_table),
        (b"cmap", cmap_table),
        (b"head", head_table),
        (b"hhea", hhea_table),
        (b"hmtx", hmtx_table),
        (b"kern", kern_table),
        (b"maxp", maxp_table),
        (b"name", name_table),
        (b"post", post_table),
    ])
}

/// Размер данных глифа с учётом выравнивания до 4 байт.
fn padded_len(glyph: &BuiltGlyph) -> u32 {
    let len = glyph.data.len() as u32;
    len + (4 - len % 4) % 4
}

/// Таблица `glyf`.
fn build_glyf_table(glyphs: &[BuiltGlyph]) -> Vec<u8> {
    let mut writer = Writer::new();
    for glyph in glyphs {
        writer.bytes(&glyph.data);
        writer.pad_to_4();
    }
    writer.bytes
}

/// Таблица `loca` (длинный формат).
///
/// Смещения обязаны совпадать с раскладкой таблицы `glyf`, поэтому длина
/// каждого глифа здесь учитывает выравнивание до 4 байт.
fn build_loca_table(glyphs: &[BuiltGlyph]) -> Vec<u8> {
    let mut writer = Writer::new();
    let mut offset = 0u32;
    for glyph in glyphs {
        writer.u32(offset);
        offset += padded_len(glyph);
    }
    writer.u32(offset);
    writer.bytes
}

/// Таблица `cmap` (форматы 4 и 12).
fn build_cmap_table(map: &BTreeMap<u32, u16>) -> Vec<u8> {
    let format4 = build_cmap_format4(map);
    let format12 = build_cmap_format12(map);

    let mut subtables = Writer::new();
    subtables.u16(0); // версия
    subtables.u16(2); // количество подтаблиц
    subtables.u16(3); // платформа Windows
    subtables.u16(1); // кодировка Unicode BMP
    subtables.u32(4 + 8 * 2); // смещение подтаблицы 4
    subtables.u16(3); // платформа Windows
    subtables.u16(10); // кодировка Unicode (полный диапазон)
    subtables.u32((4 + 8 * 2 + format4.len()) as u32);

    let mut result = subtables.bytes;
    result.extend_from_slice(&format4);
    result.extend_from_slice(&format12);
    result
}

/// Формат 4 `cmap` (диапазон BMP).
fn build_cmap_format4(map: &BTreeMap<u32, u16>) -> Vec<u8> {
    let mut codes: Vec<(u32, u16)> = map
        .iter()
        .filter(|(cp, _)| **cp <= 0xFFFF)
        .map(|(cp, gid)| (*cp, *gid))
        .collect();
    codes.sort_by_key(|(cp, _)| *cp);

    // Группируем последовательные коды с последовательными глифами.
    let mut segments: Vec<(u32, u32, i32)> = Vec::new(); // (start, end, delta)
    for (code, glyph) in codes {
        if let Some(last) = segments.last_mut() {
            if code == last.1 + 1 && glyph as i32 == (last.1 as i32 + last.2 + 1) {
                last.1 = code;
                continue;
            }
        }
        let delta = glyph as i32 - code as i32;
        segments.push((code, code, delta));
    }
    segments.push((0xFFFF, 0xFFFF, 1));

    let seg_count = segments.len();
    let mut writer = Writer::new();
    writer.u16(4);
    writer.u16((16 + seg_count * 8) as u16);
    writer.u16(0); // язык
    writer.u16((seg_count * 2) as u16);
    let search_range = 2 * (1u32 << (seg_count as f32).log2().floor() as u32);
    writer.u16(search_range as u16);
    writer.u16(((search_range / 2).trailing_zeros()) as u16);
    writer.u16((seg_count as u32 * 2 - search_range) as u16);

    for (_, end, _) in &segments {
        writer.u16(*end as u16);
    }
    writer.u16(0); // reservedPad
    for (start, _, _) in &segments {
        writer.u16(*start as u16);
    }
    for (_, _, delta) in &segments {
        writer.u16(*delta as u16);
    }
    for _ in &segments {
        writer.u16(0); // idRangeOffset
    }
    writer.bytes
}

/// Формат 12 `cmap` (полный диапазон Unicode).
fn build_cmap_format12(map: &BTreeMap<u32, u16>) -> Vec<u8> {
    let mut codes: Vec<(u32, u16)> = map.iter().map(|(cp, gid)| (*cp, *gid)).collect();
    codes.sort_by_key(|(cp, _)| *cp);

    let mut groups: Vec<(u32, u32, u32)> = Vec::new();
    for (code, glyph) in codes {
        if let Some(last) = groups.last_mut() {
            if code == last.1 + 1 && glyph as u32 == last.2 + (code - last.0) {
                last.1 = code;
                continue;
            }
        }
        groups.push((code, code, glyph as u32));
    }

    let mut writer = Writer::new();
    writer.u16(12);
    writer.u16(0);
    writer.u32(16 + groups.len() as u32 * 12);
    writer.u32(0); // язык
    writer.u32(groups.len() as u32);
    for (start, end, gid) in groups {
        writer.u32(start);
        writer.u32(end);
        writer.u32(gid);
    }
    writer.bytes
}

/// Таблица `hmtx`.
fn build_hmtx_table(glyphs: &[BuiltGlyph]) -> Vec<u8> {
    let mut writer = Writer::new();
    for glyph in glyphs {
        writer.u16(glyph.advance_width);
        writer.i16(glyph.lsb);
    }
    writer.bytes
}

/// Таблица `kern` (формат 0).
fn build_kern_table(project: &FontProject, map: &BTreeMap<u32, u16>) -> Vec<u8> {
    let mut pairs: Vec<(u16, u16, i16)> = Vec::new();
    for pair in &project.kerning {
        if let (Some(left), Some(right)) = (map.get(&pair.left), map.get(&pair.right)) {
            if pair.value != 0 {
                pairs.push((*left, *right, pair.value));
            }
        }
    }
    pairs.sort_by_key(|(l, r, _)| (*l, *r));

    let mut writer = Writer::new();
    writer.u16(0); // версия
    writer.u16(1); // количество подтаблиц

    writer.u16(0); // версия подтаблицы
    writer.u16((14 + pairs.len() * 6) as u16);
    writer.u16(0x0001); // horizontal
    writer.u16(0); // формат 0
    writer.u16(pairs.len() as u16);
    let search_range = if pairs.is_empty() {
        0
    } else {
        6 * (1u32 << (pairs.len() as f32).log2().floor() as u32)
    };
    writer.u16(search_range as u16);
    writer.u16(((search_range / 6).trailing_zeros()) as u16);
    writer.u16((pairs.len() as u16 * 6).wrapping_sub(search_range as u16));
    for (left, right, value) in pairs {
        writer.u16(left);
        writer.u16(right);
        writer.i16(value);
    }
    writer.bytes
}

/// Таблица `name`.
fn build_name_table(project: &FontProject) -> Vec<u8> {
    let meta = &project.meta;
    let full_name = format!("{} {}", meta.family_name, meta.style_name);
    let post_script = format!(
        "{}-{}",
        sanitize(&meta.family_name),
        sanitize(&meta.style_name)
    );

    let strings: Vec<(u16, String)> = vec![
        (0, meta.copyright.clone()),
        (1, meta.family_name.clone()),
        (2, meta.style_name.clone()),
        (
            3,
            format!("{}:{}:{}", meta.family_name, meta.style_name, meta.version),
        ),
        (4, full_name),
        (5, format!("Version {}", meta.version)),
        (6, post_script),
    ];

    let mut records = Writer::new();
    let mut storage = Writer::new();
    let count = strings.len();
    // Начало области строк.
    let string_offset = 6 + count * 12;
    records.u16(0); // формат
    records.u16(count as u16);
    records.u16(string_offset as u16);

    for (id, text) in &strings {
        // Смещение отсчитывается от начала области строк — так поступают
        // все распространённые шрифты и все парсеры (ttf-parser, fontdue).
        let offset = storage.len();
        storage.utf16be_string(text);
        let length = storage.len() - offset;
        records.u16(3); // платформа Windows
        records.u16(1); // кодировка UCS-2
        records.u16(0x0409); // английский
        records.u16(*id);
        records.u16(length as u16);
        records.u16(offset as u16);
    }

    let mut result = records.bytes;
    result.extend_from_slice(&storage.bytes);
    result
}

/// Убирает из строки символы, недопустимые в PostScript-имени.
fn sanitize(text: &str) -> String {
    let cleaned: String = text
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if cleaned.is_empty() {
        "Untitled".to_string()
    } else {
        cleaned
    }
}

/// Таблица `OS/2` (версия 4).
fn build_os2_table(
    project: &FontProject,
    map: &BTreeMap<u32, u16>,
    glyphs: &[PreparedGlyph],
) -> Vec<u8> {
    let meta = &project.meta;
    let mut writer = Writer::new();
    writer.u16(4); // версия
    let average = if glyphs.is_empty() {
        0
    } else {
        (glyphs.iter().map(|g| g.advance_width as i32).sum::<i32>() / glyphs.len() as i32) as i16
    };
    writer.i16(average);
    writer.u16(400); // usWeightClass
    writer.u16(5); // usWidthClass
    writer.u16(0); // fsType: устанавливаемый шрифт

    writer.i16(650); // ySubscriptXSize
    writer.i16(600); // ySubscriptYSize
    writer.i16(0); // ySubscriptXOffset
    writer.i16(75); // ySubscriptYOffset
    writer.i16(650); // ySuperscriptXSize
    writer.i16(600); // ySuperscriptYSize
    writer.i16(0); // ySuperscriptXOffset
    writer.i16(350); // ySuperscriptYOffset
    writer.i16(50); // yStrikeoutSize
    writer.i16((meta.x_height as i32 / 2) as i16); // yStrikeoutPosition
    writer.i16(0); // sFamilyClass

    // panose: моноширинный sans
    writer.bytes(&[2, 11, 5, 4, 2, 2, 4, 2, 2, 4]);
    writer.u32(1); // ulUnicodeRange1: Basic Latin
    writer.u32(0);
    writer.u32(0);
    writer.u32(0);
    writer.bytes(b"FLRD"); // achVendID

    writer.u16(0x0040); // fsSelection: REGULAR
    let first = map.keys().next().copied().unwrap_or(0xFFFF);
    let last = map.keys().next_back().copied().unwrap_or(0xFFFF);
    writer.u16(first.min(0xFFFF) as u16);
    writer.u16(last.min(0xFFFF) as u16);

    writer.i16(meta.ascender);
    writer.i16(meta.descender);
    writer.i16(meta.line_gap);
    writer.u16(meta.ascender.max(0) as u16);
    writer.u16(meta.descender.min(0).unsigned_abs());
    writer.u32(1); // ulCodePageRange1: Latin-1
    writer.u32(0);
    writer.i16(meta.x_height as i16);
    writer.i16(meta.cap_height as i16);
    writer.u16(0); // usDefaultChar
    writer.u16(32); // usBreakChar
    writer.u16(1); // usMaxContext
    writer.bytes
}

/// Таблица `post` (версия 3.0).
fn build_post_table(project: &FontProject) -> Vec<u8> {
    let mut writer = Writer::new();
    writer.u32(0x00030000);
    writer.u32(0); // italicAngle
    writer.i16(-100); // underlinePosition
    writer.i16(50); // underlineThickness
    writer.u32(0); // isFixedPitch
    writer.u32(0); // minMemType42
    writer.u32(0); // maxMemType42
    writer.u32(0); // minMemType1
    writer.u32(0); // maxMemType1
    let _ = project;
    writer.bytes
}

/// Таблица `head`.
fn build_head_table(
    _project: &FontProject,
    bbox: (i16, i16, i16, i16),
    upem: u16,
    long_loca: bool,
) -> Vec<u8> {
    let mut writer = Writer::new();
    writer.u32(0x00010000); // версия
    writer.u32(0x00010000); // fontRevision 1.0
    writer.u32(0); // checkSumAdjustment — заполняется позже
    writer.u32(0x5F0F3CF5); // magicNumber
    writer.u16(0x000B); // flags
    writer.u16(upem);
    writer.u64(0); // created
    writer.u64(0); // modified
    writer.i16(bbox.0);
    writer.i16(bbox.1);
    writer.i16(bbox.2);
    writer.i16(bbox.3);
    writer.u16(0); // macStyle
    writer.u16(8); // lowestRecPPEM
    writer.i16(2); // fontDirectionHint
    writer.i16(i16::from(long_loca)); // indexToLocFormat
    writer.i16(0); // glyphDataFormat
    writer.bytes
}

/// Таблица `hhea`.
fn build_hhea_table(
    project: &FontProject,
    advance_max: u16,
    min_lsb: i16,
    min_rsb: i16,
    max_extent: i16,
    num_glyphs: u16,
) -> Vec<u8> {
    let meta = &project.meta;
    let mut writer = Writer::new();
    writer.u32(0x00010000);
    writer.i16(meta.ascender);
    writer.i16(meta.descender);
    writer.i16(meta.line_gap);
    writer.u16(advance_max);
    writer.i16(min_lsb);
    writer.i16(min_rsb);
    writer.i16(max_extent);
    writer.i16(1); // caretSlopeRise
    writer.i16(0); // caretSlopeRun
    writer.i16(0); // caretOffset
    writer.i16(0); // reserved
    writer.i16(0);
    writer.i16(0);
    writer.i16(0);
    writer.i16(0); // metricDataFormat
    writer.u16(num_glyphs);
    writer.bytes
}

/// Таблица `maxp` (версия 1.0).
fn build_maxp_table(num_glyphs: u16, max_points: u16, max_contours: u16) -> Vec<u8> {
    let mut writer = Writer::new();
    writer.u32(0x00010000);
    writer.u16(num_glyphs);
    writer.u16(max_points);
    writer.u16(max_contours);
    writer.u16(0); // maxCompositePoints
    writer.u16(0); // maxCompositeContours
    writer.u16(2); // maxZones
    writer.u16(0); // maxTwilightPoints
    writer.u16(0); // maxStorage
    writer.u16(0); // maxFunctionDefs
    writer.u16(0); // maxInstructionDefs
    writer.u16(0); // maxStackElements
    writer.u16(0); // maxSizeOfInstructions
    writer.u16(0); // maxComponentElements
    writer.u16(0); // maxComponentDepth
    writer.bytes
}

/// Таблица `maxp` (версия 0.5) — для шрифтов с контурами CFF.
fn build_maxp_cff_table(num_glyphs: u16) -> Vec<u8> {
    let mut writer = Writer::new();
    writer.u32(0x00005000);
    writer.u16(num_glyphs);
    writer.bytes
}

/// Собирает sfnt-файл из таблиц.
pub(crate) fn assemble(version: u32, tables: Vec<Table>) -> Vec<u8> {
    let num_tables = tables.len();
    let mut sorted = tables;
    sorted.sort_by(|a, b| a.0.cmp(b.0));

    let entry_selector = {
        let mut value = 0u32;
        while (1 << (value + 1)) <= num_tables as u32 {
            value += 1;
        }
        value
    };
    let search_range = 16 * (1 << entry_selector);
    let range_shift = num_tables as u32 * 16 - search_range;

    let directory_size = 12 + num_tables * 16;
    let mut offsets = Vec::with_capacity(num_tables);
    let mut cursor = directory_size as u32;
    for (_, data) in &sorted {
        offsets.push(cursor);
        cursor += data.len() as u32;
        while cursor % 4 != 0 {
            cursor += 1;
        }
    }

    let mut font = Writer::new();
    font.u32(version);
    font.u16(num_tables as u16);
    font.u16(search_range as u16);
    font.u16(entry_selector as u16);
    font.u16(range_shift as u16);

    for (index, (tag, data)) in sorted.iter().enumerate() {
        font.bytes(&tag[..]);
        font.u32(checksum(data));
        font.u32(offsets[index]);
        font.u32(data.len() as u32);
    }

    for (index, (_, data)) in sorted.iter().enumerate() {
        while font.len() < offsets[index] as usize {
            font.u8(0);
        }
        font.bytes(data);
    }

    let mut result = font.bytes;

    // Контрольная сумма файла и checkSumAdjustment в таблице `head`.
    if let Some(head_index) = sorted.iter().position(|(tag, _)| *tag == b"head") {
        let adjustment_offset = offsets[head_index] as usize + 8;
        let sum = checksum(&result);
        let adjustment = 0xB1B0_AFBAu32.wrapping_sub(sum);
        result[adjustment_offset..adjustment_offset + 4].copy_from_slice(&adjustment.to_be_bytes());
    }

    result
}

/// Точки глифа в удобном виде (используется в тестах и инструментах).
pub fn glyph_points(contours: &[Contour]) -> Vec<GlyphPoint> {
    contours
        .iter()
        .flat_map(|c| normalize_contour(&c.points))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fontlord_core::brush::{BrushKind, BrushSettings};
    use fontlord_core::model::{Stroke, StrokePoint};

    fn sample_project() -> FontProject {
        let mut project = FontProject::new("Test Font", "Regular");
        let mut glyph = Glyph::new(0x0041, "A", 600);
        let mut stroke = Stroke::new(
            1,
            BrushKind::Marker,
            BrushSettings::for_kind(BrushKind::Marker),
        );
        stroke.closed = true;
        stroke.points = vec![
            StrokePoint::new(100.0, 0.0, 0.8),
            StrokePoint::new(500.0, 0.0, 0.8),
            StrokePoint::new(500.0, 700.0, 0.8),
            StrokePoint::new(100.0, 700.0, 0.8),
        ];
        glyph.strokes.push(stroke);
        project.glyphs.insert(0x0041, glyph);
        project
    }

    #[test]
    fn builds_valid_truetype() {
        let bytes = build_truetype(&sample_project()).unwrap();
        assert!(bytes.len() > 1000);
        assert_eq!(&bytes[0..4], &0x0001_0000u32.to_be_bytes());
    }

    #[test]
    fn rejects_invalid_upem() {
        let mut project = sample_project();
        project.meta.units_per_em = 0;
        assert!(build_truetype(&project).is_err());
    }

    #[test]
    fn builds_every_format() {
        let project = sample_project();
        for format in FontFormat::all() {
            let bytes = build_font(&project, format).unwrap();
            assert!(!bytes.is_empty(), "формат {format:?} пуст");
        }
    }

    #[test]
    fn format_extension_roundtrip() {
        for format in FontFormat::all() {
            assert_eq!(FontFormat::from_extension(format.extension()), Some(format));
        }
        assert_eq!(FontFormat::from_extension("TTF"), Some(FontFormat::Ttf));
        assert_eq!(
            FontFormat::from_extension(".woff2"),
            Some(FontFormat::Woff2)
        );
        assert_eq!(FontFormat::from_extension("otf"), Some(FontFormat::Otf));
        assert_eq!(FontFormat::from_extension("txt"), None);
    }
}
