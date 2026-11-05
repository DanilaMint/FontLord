//! Движок кистей: превращение растра (штриха) в векторные полигоны.
//!
//! Штрих — это оцифрованная пользователем линия с параметром нажима. Движок:
//!
//! 1. очищает точки от шума и сглаживает осевую линию;
//! 2. пересемплирует её с равномерным шагом;
//! 3. вычисляет полуширину в каждой точке согласно модели кисти;
//! 4. строит смещённые границы и замыкает их в полигон(ы).
//!
//! Результат — набор полигонов с корректной ориентацией: внешние контуры имеют
//! положительную ориентированную площадь, «дырки» — отрицательную. Благодаря
//! этому правило ненулевой намотки в TrueType корректно склеивает перекрывающиеся
//! мазки и вырезает внутренности замкнутых контуров.

use crate::brush::{BrushKind, BrushSettings};
use crate::geometry::{
    Point, dedup, ensure_ccw, ensure_cw, polygon_signed_area, polyline_length, resample,
    smooth_pressures, smoothstep,
};
use crate::model::{Stroke, StrokePoint};

/// Полигон, полученный из штриха.
#[derive(Debug, Clone, PartialEq)]
pub struct StrokePolygon {
    /// Вершины полигона (замкнутого).
    pub points: Vec<Point>,
    /// Признак того, что полигон является внутренностью (дыркой).
    pub hole: bool,
}

/// Минимальное расстояние между точками штриха (шум ввода).
const MIN_POINT_DISTANCE: f32 = 0.75;

/// Сглаживает осевую линию штриха (Катмулл–Ром по позициям, линейно по нажиму).
fn smooth_centerline(
    points: &[StrokePoint],
    closed: bool,
    subdivisions: usize,
) -> Vec<StrokePoint> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let n = points.len();
    let index = |i: isize| -> StrokePoint {
        if closed {
            let len = n as isize;
            let wrapped = ((i % len) + len) % len;
            points[wrapped as usize]
        } else {
            points[i.clamp(0, n as isize - 1) as usize]
        }
    };

    let segments = if closed { n } else { n - 1 };
    let mut out = Vec::with_capacity(n * subdivisions);
    for i in 0..segments {
        let p0 = index(i as isize - 1);
        let p1 = index(i as isize);
        let p2 = index(i as isize + 1);
        let p3 = index(i as isize + 2);
        for s in 0..subdivisions {
            let t = s as f32 / subdivisions as f32;
            let t2 = t * t;
            let t3 = t2 * t;
            let x = p0.x * (-0.5 * t3 + t2 - 0.5 * t)
                + p1.x * (1.5 * t3 - 2.5 * t2 + 1.0)
                + p2.x * (-1.5 * t3 + 2.0 * t2 + 0.5 * t)
                + p3.x * (0.5 * t3 - 0.5 * t2);
            let y = p0.y * (-0.5 * t3 + t2 - 0.5 * t)
                + p1.y * (1.5 * t3 - 2.5 * t2 + 1.0)
                + p2.y * (-1.5 * t3 + 2.0 * t2 + 0.5 * t)
                + p3.y * (0.5 * t3 - 0.5 * t2);
            let pressure = p1.pressure + (p2.pressure - p1.pressure) * t;
            out.push(StrokePoint { x, y, pressure });
        }
    }
    if !closed {
        out.push(*points.last().unwrap());
    }
    out
}

/// Профиль толщины кисти: возвращает долю от номинальной ширины.
fn width_profile(kind: BrushKind, settings: &BrushSettings, pressure: f32) -> f32 {
    let p = pressure.clamp(0.0, 1.0);
    match kind {
        BrushKind::Pen | BrushKind::Soft => {
            let shaped = p.powf(settings.pressure_gamma.max(0.05));
            settings.min_width_ratio + (1.0 - settings.min_width_ratio) * shaped
        }
        BrushKind::Marker => 0.85 + 0.15 * p,
        BrushKind::BroadNib | BrushKind::Eraser => 1.0,
    }
}

/// Коэффициент сужения концов штриха.
fn taper_factor(settings: &BrushSettings, t: f32) -> f32 {
    let mut factor: f32 = 1.0;
    if settings.taper_in > 0.0 {
        factor *= smoothstep(t / settings.taper_in);
    }
    if settings.taper_out > 0.0 {
        factor *= smoothstep((1.0 - t) / settings.taper_out);
    }
    factor.clamp(0.0, 1.0)
}

/// Единичный вектор пера для каллиграфической кисти.
fn nib_vector(settings: &BrushSettings) -> Point {
    Point::new(settings.angle.cos(), settings.angle.sin())
}

/// Диск (для штриха в одну точку).
fn disc(center: Point, radius: f32, segments: usize) -> Vec<Point> {
    let radius = radius.max(0.05);
    let segments = segments.max(8);
    (0..segments)
        .map(|i| {
            let angle = i as f32 / segments as f32 * std::f32::consts::TAU;
            Point::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            )
        })
        .collect()
}

/// Превращает штрих в набор полигонов.
pub fn stroke_to_polygons(stroke: &Stroke) -> Vec<StrokePolygon> {
    let settings = stroke.settings;
    let kind = stroke.kind;

    // 1. Очистка от шума.
    let raw: Vec<Point> = stroke.points.iter().map(|p| Point::new(p.x, p.y)).collect();
    let raw = dedup(&raw, MIN_POINT_DISTANCE);
    if raw.is_empty() {
        return Vec::new();
    }

    let mut pressures: Vec<f32> = stroke
        .points
        .iter()
        .map(|p| p.pressure.clamp(0.0, 1.0))
        .collect();
    // Очистка шума нажима должна идти по той же длине, что и осевая линия.
    if pressures.len() > raw.len() {
        pressures.truncate(raw.len());
    }
    if pressures.len() < raw.len() {
        pressures.resize(raw.len(), pressures.last().copied().unwrap_or(0.6));
    }
    pressures = smooth_pressures(&pressures, 5);

    let points: Vec<StrokePoint> = raw
        .iter()
        .zip(pressures.iter())
        .map(|(p, pr)| StrokePoint::new(p.x, p.y, *pr))
        .collect();

    let half_width = (settings.width * 0.5).max(0.5);

    // 2. Штрих в одну точку — рисуем точку.
    if points.len() == 1 {
        let p = points[0];
        let radius = half_width * width_profile(kind, &settings, p.pressure);
        return vec![StrokePolygon {
            points: disc(Point::new(p.x, p.y), radius, 24),
            hole: false,
        }];
    }

    let closed = stroke.closed;
    let mut samples = points.clone();
    if closed {
        // Замыкаем кольцо: последняя точка не должна дублировать первую.
        if let (Some(first), Some(last)) = (samples.first().copied(), samples.last().copied()) {
            if Point::new(first.x, first.y).distance(Point::new(last.x, last.y))
                < MIN_POINT_DISTANCE * 2.0
            {
                samples.pop();
            }
        }
        if samples.len() < 3 {
            samples = points.clone();
        }
    }

    // 3. Сглаживание и пересемплирование.
    let smoothed = smooth_centerline(&samples, closed, 8);
    let step = (settings.width / 14.0).max(1.0);
    let (positions, press) = resample(
        &smoothed
            .iter()
            .map(|p| Point::new(p.x, p.y))
            .collect::<Vec<_>>(),
        &smoothed.iter().map(|p| p.pressure).collect::<Vec<_>>(),
        step,
    );
    if positions.len() < 2 {
        return Vec::new();
    }

    let total_length = polyline_length(&positions);
    if total_length <= f32::EPSILON {
        return Vec::new();
    }

    // 4. Полуширина в каждой точке.
    let mut widths = Vec::with_capacity(positions.len());
    let mut travelled = 0.0f32;
    for (i, _) in positions.iter().enumerate() {
        let t = (travelled / total_length).clamp(0.0, 1.0);
        let taper = if closed {
            1.0
        } else {
            taper_factor(&settings, t)
        };
        let profile = width_profile(kind, &settings, press[i]);
        widths.push(half_width * profile * taper);
        if i + 1 < positions.len() {
            travelled += positions[i].distance(positions[i + 1]);
        }
    }

    // 5. Построение полигонов.
    if kind == BrushKind::BroadNib {
        broad_nib_polygons(&positions, &settings, closed, &widths)
    } else {
        offset_polygons(&positions, &widths, closed)
    }
}

/// Полигоны для каллиграфической кисти (перо фиксированной ширины под углом).
fn broad_nib_polygons(
    positions: &[Point],
    settings: &BrushSettings,
    closed: bool,
    widths: &[f32],
) -> Vec<StrokePolygon> {
    let nib = nib_vector(settings);
    let ring_a: Vec<Point> = positions
        .iter()
        .enumerate()
        .map(|(i, &p)| p + nib * (widths[i] * 0.5))
        .collect();
    let ring_b: Vec<Point> = positions
        .iter()
        .enumerate()
        .map(|(i, &p)| p - nib * (widths[i] * 0.5))
        .collect();

    if closed {
        let area_a = polygon_signed_area(&ring_a).abs();
        let area_b = polygon_signed_area(&ring_b).abs();
        let (outer, inner) = if area_a >= area_b {
            (ring_a, ring_b)
        } else {
            (ring_b, ring_a)
        };
        let mut outer = outer;
        let mut inner = inner;
        ensure_ccw(&mut outer);
        ensure_cw(&mut inner);
        return vec![
            StrokePolygon {
                points: outer,
                hole: false,
            },
            StrokePolygon {
                points: inner,
                hole: true,
            },
        ];
    }

    let mut polygon = ring_a;
    polygon.extend(ring_b.iter().rev().copied());
    // Ориентация не должна зависеть от направления рисования: иначе
    // перекрывающиеся мазки вычитались бы друг из друга.
    ensure_ccw(&mut polygon);
    vec![StrokePolygon {
        points: polygon,
        hole: false,
    }]
}

/// Полигоны для кистей с переменной толщиной (смещение по нормали).
fn offset_polygons(positions: &[Point], widths: &[f32], closed: bool) -> Vec<StrokePolygon> {
    let n = positions.len();
    let mut left = Vec::with_capacity(n);
    let mut right = Vec::with_capacity(n);

    for i in 0..n {
        let tangent = if closed {
            let prev = positions[(i + n - 1) % n];
            let next = positions[(i + 1) % n];
            (next - prev).normalized()
        } else if i == 0 {
            (positions[1] - positions[0]).normalized()
        } else if i + 1 == n {
            (positions[n - 1] - positions[n - 2]).normalized()
        } else {
            (positions[i + 1] - positions[i - 1]).normalized()
        };
        let normal = tangent.perpendicular();
        let w = widths[i];
        left.push(positions[i] + normal * w);
        right.push(positions[i] - normal * w);
    }

    if closed {
        let area_left = polygon_signed_area(&left).abs();
        let area_right = polygon_signed_area(&right).abs();
        let (mut outer, mut inner) = if area_left >= area_right {
            (left, right)
        } else {
            (right, left)
        };
        ensure_ccw(&mut outer);
        ensure_cw(&mut inner);
        return vec![
            StrokePolygon {
                points: outer,
                hole: false,
            },
            StrokePolygon {
                points: inner,
                hole: true,
            },
        ];
    }

    let mut polygon = left;
    polygon.extend(right.iter().rev().copied());
    // Ориентация не должна зависеть от направления рисования: иначе
    // перекрывающиеся мазки вычитались бы друг из друга.
    ensure_ccw(&mut polygon);
    vec![StrokePolygon {
        points: polygon,
        hole: false,
    }]
}

/// Удаляет штрихи, которых касается путь ластика.
///
/// Возвращает список идентификаторов удалённых штрихов.
pub fn erase_strokes(strokes: &mut Vec<Stroke>, path: &[Point], radius: f32) -> Vec<u32> {
    if path.is_empty() {
        return Vec::new();
    }
    let mut removed = Vec::new();
    strokes.retain(|stroke| {
        let touched = stroke.points.iter().any(|p| {
            let point = Point::new(p.x, p.y);
            path.iter().any(|q| point.distance(*q) <= radius)
        });
        if touched {
            removed.push(stroke.id);
        }
        !touched
    });
    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brush::BrushSettings;

    fn stroke_with(points: &[(f32, f32)]) -> Stroke {
        let mut stroke = Stroke::new(1, BrushKind::Pen, BrushSettings::default());
        stroke.points = points
            .iter()
            .map(|(x, y)| StrokePoint::new(*x, *y, 0.8))
            .collect();
        stroke
    }

    #[test]
    fn horizontal_line_produces_positive_area() {
        let stroke = stroke_with(&[(0.0, 0.0), (200.0, 0.0)]);
        let polygons = stroke_to_polygons(&stroke);
        assert_eq!(polygons.len(), 1);
        assert!(polygon_signed_area(&polygons[0].points) > 0.0);
    }

    #[test]
    fn closed_stroke_produces_hole() {
        let mut stroke = stroke_with(&[(0.0, 0.0), (200.0, 0.0), (200.0, 200.0), (0.0, 200.0)]);
        stroke.closed = true;
        let polygons = stroke_to_polygons(&stroke);
        assert_eq!(polygons.len(), 2);
        assert!(!polygons[0].hole);
        assert!(polygons[1].hole);
        assert!(polygon_signed_area(&polygons[0].points) > 0.0);
        assert!(polygon_signed_area(&polygons[1].points) < 0.0);
    }

    #[test]
    fn opposite_directions_keep_same_orientation() {
        let forward = stroke_with(&[(0.0, 0.0), (200.0, 0.0)]);
        let backward = stroke_with(&[(200.0, 0.0), (0.0, 0.0)]);
        let a = stroke_to_polygons(&forward);
        let b = stroke_to_polygons(&backward);
        assert!(polygon_signed_area(&a[0].points) > 0.0);
        assert!(polygon_signed_area(&b[0].points) > 0.0);
    }

    #[test]
    fn eraser_removes_touched_strokes() {
        let mut strokes = vec![stroke_with(&[(0.0, 0.0), (50.0, 0.0)]), {
            let mut s = stroke_with(&[(500.0, 0.0), (600.0, 0.0)]);
            s.id = 2;
            s
        }];
        let removed = erase_strokes(&mut strokes, &[Point::new(10.0, 0.0)], 20.0);
        assert_eq!(removed, vec![1]);
        assert_eq!(strokes.len(), 1);
    }
}
