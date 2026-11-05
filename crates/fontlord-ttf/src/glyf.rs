//! Кодирование таблицы `glyf`: простые и составные глифы TrueType.

use fontlord_core::model::ContourPoint;

use crate::writer::Writer;

/// Точка глифа в целочисленных шрифтовых единицах.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphPoint {
    /// Координата X.
    pub x: i16,
    /// Координата Y.
    pub y: i16,
    /// Признак точки, лежащей на контуре.
    pub on_curve: bool,
}

/// Границы глифа.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoundingBox {
    /// Минимальная X.
    pub x_min: i16,
    /// Минимальная Y.
    pub y_min: i16,
    /// Максимальная X.
    pub x_max: i16,
    /// Максимальная Y.
    pub y_max: i16,
}

impl BoundingBox {
    /// Расширяет границы точкой.
    pub fn extend(&mut self, point: GlyphPoint) {
        self.x_min = self.x_min.min(point.x);
        self.y_min = self.y_min.min(point.y);
        self.x_max = self.x_max.max(point.x);
        self.y_max = self.y_max.max(point.y);
    }

    /// Признак пустых границ.
    pub fn is_empty(&self) -> bool {
        self.x_min > self.x_max
    }
}

/// Данные простого глифа.
#[derive(Debug, Clone, Default)]
pub struct SimpleGlyph {
    /// Контуры глифа.
    pub contours: Vec<Vec<GlyphPoint>>,
    /// Границы глифа.
    pub bbox: BoundingBox,
}

/// Приводит контур к каноническому виду TrueType.
///
/// Правила:
///
/// * контур начинается с точки на кривой;
/// * между двумя соседними управляющими точками вставляется подразумеваемая
///   точка на кривой (середина отрезка).
pub fn normalize_contour(points: &[ContourPoint]) -> Vec<GlyphPoint> {
    let mut pts: Vec<GlyphPoint> = points
        .iter()
        .map(|p| GlyphPoint {
            x: p.x.round() as i16,
            y: p.y.round() as i16,
            on_curve: p.on_curve,
        })
        .collect();

    if pts.len() < 2 {
        return pts;
    }

    // Если точек на кривой нет — создаём подразумеваемые середины.
    if !pts.iter().any(|p| p.on_curve) {
        let count = pts.len();
        let mut result = Vec::with_capacity(count * 2);
        for i in 0..count {
            let current = pts[i];
            let next = pts[(i + 1) % count];
            result.push(GlyphPoint {
                x: ((current.x as i32 + next.x as i32) / 2) as i16,
                y: ((current.y as i32 + next.y as i32) / 2) as i16,
                on_curve: true,
            });
            result.push(current);
        }
        return result;
    }

    // Ротируем так, чтобы первая точка лежала на кривой.
    if !pts[0].on_curve {
        if let Some(position) = pts.iter().position(|p| p.on_curve) {
            pts.rotate_left(position);
        }
    }

    // Вставляем подразумеваемые точки между соседними управляющими.
    let mut result: Vec<GlyphPoint> = Vec::with_capacity(pts.len() * 2);
    for i in 0..pts.len() {
        let current = pts[i];
        let next = pts[(i + 1) % pts.len()];
        result.push(current);
        if !current.on_curve && !next.on_curve {
            result.push(GlyphPoint {
                x: ((current.x as i32 + next.x as i32) / 2) as i16,
                y: ((current.y as i32 + next.y as i32) / 2) as i16,
                on_curve: true,
            });
        }
    }
    result
}

/// Кодирует простой глиф в формат таблицы `glyf`.
pub fn encode_simple_glyph(glyph: &SimpleGlyph) -> Vec<u8> {
    if glyph.contours.is_empty() {
        return Vec::new();
    }

    let mut writer = Writer::new();
    writer.i16(glyph.contours.len() as i16);
    writer.i16(glyph.bbox.x_min);
    writer.i16(glyph.bbox.y_min);
    writer.i16(glyph.bbox.x_max);
    writer.i16(glyph.bbox.y_max);

    // endPtsOfContours хранит индекс последней точки каждого контура,
    // а не количество точек в нём.
    let mut total_points = 0u16;
    for contour in &glyph.contours {
        total_points += contour.len() as u16;
        writer.u16(total_points - 1);
    }
    writer.u16(0); // длина инструкций

    // Флаги и координаты.
    let mut flags: Vec<u8> = Vec::new();
    let mut x_data: Vec<u8> = Vec::new();
    let mut y_data: Vec<u8> = Vec::new();
    let mut previous_x = 0i32;
    let mut previous_y = 0i32;

    for contour in &glyph.contours {
        for point in contour {
            let dx = point.x as i32 - previous_x;
            let dy = point.y as i32 - previous_y;
            previous_x = point.x as i32;
            previous_y = point.y as i32;

            let mut flag = 0u8;
            if point.on_curve {
                flag |= 0x01;
            }

            if dx == 0 {
                flag |= 0x10; // X_SAME
            } else if (-255..=255).contains(&dx) {
                flag |= 0x02; // X_SHORT
                if dx > 0 {
                    flag |= 0x10; // положительное смещение
                }
                // Для короткого смещения хранится модуль значения,
                // а знак задаётся битом X_SAME_OR_POSITIVE.
                x_data.push(dx.unsigned_abs() as u8);
            } else {
                x_data.extend_from_slice(&(dx as i16).to_be_bytes());
            }

            if dy == 0 {
                flag |= 0x20; // Y_SAME
            } else if (-255..=255).contains(&dy) {
                flag |= 0x04; // Y_SHORT
                if dy > 0 {
                    flag |= 0x20;
                }
                // Для короткого смещения хранится модуль значения,
                // а знак задаётся битом Y_SAME_OR_POSITIVE.
                y_data.push(dy.unsigned_abs() as u8);
            } else {
                y_data.extend_from_slice(&(dy as i16).to_be_bytes());
            }

            flags.push(flag);
        }
    }

    // Сжатие повторяющихся флагов.
    let mut index = 0usize;
    while index < flags.len() {
        let flag = flags[index];
        let mut repeat = 0usize;
        while index + repeat + 1 < flags.len() && flags[index + repeat + 1] == flag {
            repeat += 1;
        }
        if repeat > 0 {
            writer.u8(flag | 0x08);
            writer.u8(repeat as u8);
            index += repeat + 1;
        } else {
            writer.u8(flag);
            index += 1;
        }
    }

    writer.bytes(&x_data);
    writer.bytes(&y_data);
    writer.bytes
}

/// Кодирует составной глиф из ссылок на другие глифы.
///
/// `components` — список `(glyph_id, dx, dy)`.
pub fn encode_composite_glyph(components: &[(u16, i16, i16)], bbox: BoundingBox) -> Vec<u8> {
    let mut writer = Writer::new();
    writer.i16(-1); // составной глиф
    writer.i16(bbox.x_min);
    writer.i16(bbox.y_min);
    writer.i16(bbox.x_max);
    writer.i16(bbox.y_max);

    for (index, (glyph_id, dx, dy)) in components.iter().enumerate() {
        let mut flags: u16 = 0x0001; // ARGS_ARE_XY_VALUES
        flags |= 0x0002; // ARGS_ARE_WORDS
        if index + 1 < components.len() {
            flags |= 0x0020; // MORE_COMPONENTS
        }
        writer.u16(flags);
        writer.u16(*glyph_id);
        writer.i16(*dx);
        writer.i16(*dy);
    }
    writer.u16(0); // количество инструкций
    writer.bytes
}

/// Вычисляет границы контуров.
pub fn bbox_of(contours: &[Vec<GlyphPoint>]) -> BoundingBox {
    let mut bbox = BoundingBox {
        x_min: i16::MAX,
        y_min: i16::MAX,
        x_max: i16::MIN,
        y_max: i16::MIN,
    };
    for contour in contours {
        for point in contour {
            bbox.extend(*point);
        }
    }
    if bbox.is_empty() {
        bbox = BoundingBox::default();
    }
    bbox
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_inserts_implied_points() {
        let points = vec![
            ContourPoint {
                x: 0.0,
                y: 0.0,
                on_curve: false,
            },
            ContourPoint {
                x: 10.0,
                y: 0.0,
                on_curve: false,
            },
        ];
        let normalized = normalize_contour(&points);
        assert!(normalized.iter().any(|p| p.on_curve));
        // Две управляющие точки превращаются в четыре: две подразумеваемые
        // точки на кривой и две управляющие.
        assert_eq!(normalized.len(), 4);
        assert!(normalized[0].on_curve);
        assert!(!normalized[1].on_curve);
    }

    #[test]
    fn normalize_rotates_to_on_curve_point() {
        let points = vec![
            ContourPoint {
                x: 5.0,
                y: 0.0,
                on_curve: false,
            },
            ContourPoint {
                x: 10.0,
                y: 10.0,
                on_curve: true,
            },
            ContourPoint {
                x: 0.0,
                y: 10.0,
                on_curve: true,
            },
        ];
        let normalized = normalize_contour(&points);
        assert!(normalized[0].on_curve);
        assert_eq!(normalized.len(), 3);
    }

    #[test]
    fn encodes_empty_glyph_as_empty_data() {
        let glyph = SimpleGlyph::default();
        assert!(encode_simple_glyph(&glyph).is_empty());
    }
}
