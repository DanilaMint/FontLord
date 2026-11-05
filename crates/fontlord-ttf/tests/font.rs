//! Интеграционные тесты сборщика шрифтов.
//!
//! Готовый файл проверяется независимым парсером (`ttf-parser`) и
//! растеризатором (`fontdue`): это гарантирует, что экспортированный шрифт
//! читается сторонними программами, а не только нашим кодом.

use fontlord_core::brush::{BrushKind, BrushSettings};
use fontlord_core::model::{FontProject, Glyph, Stroke, StrokePoint};
use fontlord_ttf::build_truetype;

fn sample_project() -> FontProject {
    let mut project = FontProject::new("Fontlord Test", "Regular");
    project.meta.x_height = 500;
    project.meta.cap_height = 700;

    // Буква «A» — прямоугольный контур.
    let mut a = Glyph::new(0x0041, "A", 600);
    let mut ring = Stroke::new(
        1,
        BrushKind::Marker,
        BrushSettings::for_kind(BrushKind::Marker),
    );
    ring.closed = true;
    ring.points = vec![
        StrokePoint::new(120.0, 0.0, 0.8),
        StrokePoint::new(480.0, 0.0, 0.8),
        StrokePoint::new(480.0, 700.0, 0.8),
        StrokePoint::new(120.0, 700.0, 0.8),
    ];
    a.strokes.push(ring);
    project.glyphs.insert(0x0041, a);

    // Буква «B» — вертикальный мазок.
    let mut b = Glyph::new(0x0042, "B", 560);
    let mut stem = Stroke::new(
        1,
        BrushKind::Pen,
        BrushSettings {
            width: 90.0,
            ..BrushSettings::for_kind(BrushKind::Pen)
        },
    );
    stem.points = vec![
        StrokePoint::new(150.0, 0.0, 0.9),
        StrokePoint::new(150.0, 350.0, 0.7),
        StrokePoint::new(150.0, 700.0, 0.9),
    ];
    b.strokes.push(stem);
    project.glyphs.insert(0x0042, b);

    project
}

/// Декодирует строку таблицы `name` (Windows использует UTF-16BE).
fn decode_name(raw: &[u8]) -> String {
    if raw.len() % 2 == 0 {
        let units: Vec<u16> = raw
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16_lossy(&units);
    }
    String::from_utf8_lossy(raw).to_string()
}

#[test]
fn exported_font_parses_with_ttf_parser() {
    let bytes = build_truetype(&sample_project()).expect("шрифт собран");
    let face = ttf_parser::Face::parse(&bytes, 0).expect("шрифт разбирается");

    assert_eq!(face.units_per_em(), 1000);
    assert_eq!(face.number_of_glyphs(), 3); // .notdef + A + B
    assert_eq!(face.ascender(), 800);
    assert_eq!(face.descender(), -200);

    let tables = face.tables();
    assert!(tables.head.units_per_em > 0, "нет таблицы head");
    assert!(tables.hhea.number_of_metrics > 0, "нет таблицы hhea");
    assert!(tables.maxp.number_of_glyphs.get() > 0, "нет таблицы maxp");
    assert!(tables.hmtx.is_some(), "нет таблицы hmtx");
    assert!(tables.cmap.is_some(), "нет таблицы cmap");
    assert!(tables.glyf.is_some(), "нет таблицы glyf");
    assert!(tables.name.is_some(), "нет таблицы name");
    assert!(tables.os2.is_some(), "нет таблицы OS/2");

    let glyph_id = face.glyph_index('A').expect("символ A есть в cmap");
    assert!(glyph_id.0 > 0);

    let family = face
        .names()
        .into_iter()
        .find(|n| n.name_id == 1 && n.platform_id == ttf_parser::PlatformId::Windows)
        .map(|n| decode_name(n.name))
        .expect("имя семейства найдено");
    assert_eq!(family, "Fontlord Test");
}

#[test]
fn exported_font_rasterizes() {
    let bytes = build_truetype(&sample_project()).expect("шрифт собран");
    let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
        .expect("шрифт загружается");

    let glyph_id = font.lookup_glyph_index('A');
    assert!(glyph_id > 0);

    let (metrics, pixels) = font.rasterize('A', 64.0);
    assert!(metrics.width > 0 && metrics.height > 0);
    assert!(
        pixels.iter().any(|p| *p > 0),
        "глиф должен содержать закрашенные пиксели"
    );
}

#[test]
fn exported_font_without_glyphs_is_still_valid() {
    let project = FontProject::new("Empty", "Regular");
    let bytes = build_truetype(&project).expect("шрифт собран");
    let face = ttf_parser::Face::parse(&bytes, 0).expect("шрифт разбирается");
    assert_eq!(face.number_of_glyphs(), 1);
}
