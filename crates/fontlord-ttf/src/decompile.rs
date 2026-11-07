//! Декомпиляция готовых шрифтов: чтение файла шрифта в проект Fontlord.
//!
//! Разбор файла выполняется библиотекой `ttf-parser`, которая умеет читать
//! TrueType, OpenType (CFF), коллекции шрифтов и Web-шрифты. Контуры глифов
//! переносятся в проект как базовые контуры: их можно сразу просматривать
//! и экспортировать, а поверх них можно рисовать новые штрихи.

use fontlord_core::model::{Contour, ContourPoint, FontMeta, FontProject, Glyph};
use fontlord_core::unicode::postscript_name;
use ttf_parser::{Face, GlyphId, OutlineBuilder as OutlineSink};

use crate::FontError;

/// Отчёт о декомпиляции шрифта.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecompileReport {
    /// Имя семейства шрифта.
    pub family_name: String,
    /// Имя начертания.
    pub style_name: String,
    /// Количество глифов в проекте.
    pub glyph_count: usize,
    /// Единицы на em исходного шрифта.
    pub units_per_em: u16,
}

/// Количество сегментов, на которые дробятся кубические кривые (OpenType/CFF).
const CUBIC_STEPS: usize = 10;

/// Декомпилирует файл шрифта в проект Fontlord.
pub fn decompile_font(data: &[u8]) -> Result<(FontProject, DecompileReport), FontError> {
    let face = Face::parse(data, 0).map_err(|e| FontError::Parse(e.to_string()))?;

    let units_per_em = face.units_per_em();
    let family_name = name_of(&face, 1).unwrap_or_else(|| "Decompiled Font".to_string());
    let style_name = name_of(&face, 2).unwrap_or_else(|| "Regular".to_string());

    let meta = FontMeta {
        family_name: family_name.clone(),
        style_name: style_name.clone(),
        version: name_of(&face, 5).unwrap_or_else(|| "1.000".to_string()),
        units_per_em,
        ascender: face.ascender(),
        descender: face.descender(),
        line_gap: face.line_gap(),
        x_height: face
            .x_height()
            .unwrap_or((units_per_em as f32 * 0.5) as i16) as u16,
        cap_height: (units_per_em as f32 * 0.7) as u16,
        designer: name_of(&face, 9).unwrap_or_default(),
        copyright: name_of(&face, 0).unwrap_or_default(),
        license: name_of(&face, 14).unwrap_or_default(),
    };

    let mut project = FontProject {
        meta,
        ..FontProject::default()
    };

    for cp in 0..=0x10_FFFFu32 {
        let Some(ch) = char::from_u32(cp) else {
            continue;
        };
        let Some(glyph_id) = face.glyph_index(ch) else {
            continue;
        };
        if glyph_id.0 == 0 {
            // .notdef — служебный глиф, в проект не переносится.
            continue;
        }

        let mut builder = GlyphOutline::default();
        face.outline_glyph(glyph_id, &mut builder);
        let contours = builder.finish();
        if contours.is_empty() {
            continue;
        }

        let advance = face.glyph_hor_advance(glyph_id).unwrap_or(units_per_em / 2);
        let name = face
            .glyph_name(glyph_id)
            .filter(|value| !value.trim().is_empty())
            .map(|value| value.to_string())
            .unwrap_or_else(|| postscript_name(cp));

        let mut glyph = Glyph::new(cp, name, advance);
        glyph.base_contours = contours.clone();
        glyph.contours = contours;
        project.glyphs.insert(cp, glyph);
    }

    if project.glyphs.is_empty() {
        return Err(FontError::Parse(
            "в шрифте не найдено ни одного глифа с символом".to_string(),
        ));
    }

    let report = DecompileReport {
        family_name,
        style_name,
        glyph_count: project.glyphs.len(),
        units_per_em,
    };
    Ok((project, report))
}

/// Читает строку из таблицы `name` по идентификатору записи.
fn name_of(face: &Face, name_id: u16) -> Option<String> {
    face.names().get(name_id).and_then(|name| name.to_string())
}

/// Собирает контуры глифа из потока точек TrueType.
#[derive(Debug, Default)]
struct GlyphOutline {
    /// Готовые контуры.
    contours: Vec<Contour>,
    /// Точки текущего контура.
    current: Vec<ContourPoint>,
}

impl GlyphOutline {
    /// Закрывает текущий контур, если он накопил точки.
    fn flush(&mut self) {
        if self.current.len() < 2 {
            self.current.clear();
            return;
        }
        // Последняя точка не должна совпадать с первой: контур замыкается неявно.
        if let (Some(first), Some(last)) = (self.current.first(), self.current.last()) {
            if first.on_curve
                && last.on_curve
                && (first.x - last.x).abs() < 0.01
                && (first.y - last.y).abs() < 0.01
            {
                self.current.pop();
            }
        }
        if self.current.len() >= 2 {
            self.contours
                .push(Contour::new(std::mem::take(&mut self.current)));
        } else {
            self.current.clear();
        }
    }

    /// Возвращает собранные контуры.
    fn finish(mut self) -> Vec<Contour> {
        self.flush();
        self.contours
    }
}

impl OutlineSink for GlyphOutline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.flush();
        self.current.push(ContourPoint::on_curve(x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.current.push(ContourPoint::on_curve(x, y));
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.current.push(ContourPoint::off_curve(cx, cy));
        self.current.push(ContourPoint::on_curve(x, y));
    }

    fn curve_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        // Кубические кривые (OpenType/CFF) дробятся на отрезки: модель
        // Fontlord использует квадратичные кривые.
        let start = match self.current.last() {
            Some(point) if point.on_curve => (point.x, point.y),
            _ => return,
        };
        for step in 1..=CUBIC_STEPS {
            let t = step as f32 / CUBIC_STEPS as f32;
            let u = 1.0 - t;
            let px =
                u * u * u * start.0 + 3.0 * u * u * t * c1x + 3.0 * u * t * t * c2x + t * t * t * x;
            let py =
                u * u * u * start.1 + 3.0 * u * u * t * c1y + 3.0 * u * t * t * c2y + t * t * t * y;
            self.current.push(ContourPoint::on_curve(px, py));
        }
    }

    fn close(&mut self) {
        self.flush();
    }
}

/// Служебная функция: идентификатор глифа по коду символа.
#[allow(dead_code)]
fn glyph_of(face: &Face, codepoint: u32) -> Option<GlyphId> {
    char::from_u32(codepoint).and_then(|ch| face.glyph_index(ch))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fontlord_core::model::{FontProject, Stroke};

    /// Простой шрифт для проверки декомпиляции.
    fn sample_font() -> Vec<u8> {
        let mut project = FontProject::new("Test", "Regular");
        let mut glyph = fontlord_core::model::Glyph::new(0x0041, "A", 600);
        let mut stroke = Stroke::new(
            1,
            fontlord_core::brush::BrushKind::Marker,
            fontlord_core::brush::BrushSettings::for_kind(fontlord_core::brush::BrushKind::Marker),
        );
        stroke.closed = true;
        stroke.points = vec![
            fontlord_core::model::StrokePoint::new(100.0, 0.0, 0.8),
            fontlord_core::model::StrokePoint::new(500.0, 0.0, 0.8),
            fontlord_core::model::StrokePoint::new(500.0, 700.0, 0.8),
            fontlord_core::model::StrokePoint::new(100.0, 700.0, 0.8),
        ];
        glyph.strokes.push(stroke);
        glyph.rebuild_outline(&fontlord_core::outline::OutlineSettings::default());
        project.glyphs.insert(0x0041, glyph);
        crate::build_truetype(&project).expect("шрифт собран")
    }

    #[test]
    fn decompiles_simple_glyph() {
        let (project, report) = decompile_font(&sample_font()).expect("декомпиляция");
        assert_eq!(report.glyph_count, 1);
        assert_eq!(report.units_per_em, 1000);
        let glyph = project.glyph(0x0041).expect("глиф A");
        assert!(!glyph.contours.is_empty());
        assert_eq!(glyph.advance_width, 600);
    }

    #[test]
    fn rejects_garbage() {
        assert!(decompile_font(b"not a font").is_err());
    }
}
