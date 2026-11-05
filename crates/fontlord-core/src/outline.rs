//! Построение векторных контуров глифа по штрихам.
//!
//! Полигоны, полученные движком кистей, аппроксимируются квадратичными кривыми
//! (формат TrueType). Углы сохраняются острыми, гладкие участки аппроксимируются
//! кривыми с заданной погрешностью.

use crate::geometry::{
    Point, dedup, polygon_signed_area, simplify_rdp, turn_angle,
};
use crate::model::{Contour, ContourPoint, Glyph};
use crate::stroke::stroke_to_polygons;

/// Параметры аппроксимации контуров.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutlineSettings {
    /// Допустимая погрешность аппроксимации в шрифтовых единицах.
    pub curve_tolerance: f32,
    /// Порог угла (в градусах), ниже которого вершина считается углом и не
    /// сглаживается.
    pub corner_angle_degrees: f32,
    /// Допуск упрощения полигона перед аппроксимацией.
    pub simplify_epsilon: f32,
}

impl Default for OutlineSettings {
    fn default() -> Self {
        Self {
            curve_tolerance: 1.0,
            corner_angle_degrees: 100.0,
            simplify_epsilon: 0.4,
        }
    }
}

/// Строит контуры глифа по его штрихам.
pub fn build_glyph_outline(glyph: &Glyph, settings: &OutlineSettings) -> Vec<Contour> {
    let mut contours = Vec::new();

    for stroke in &glyph.strokes {
        if stroke.kind.is_eraser() {
            // Ластик работает на уровне штрихов и не оставляет формы.
            continue;
        }
        for polygon in stroke_to_polygons(stroke) {
            let points = fit_polygon(&polygon.points, settings);
            if points.len() < 3 {
                continue;
            }
            let mut contour = Contour::new(points);
            // Ориентация: внешние контуры против часовой стрелки, дырки — по часовой.
            if polygon.hole {
                if contour.signed_area() > 0.0 {
                    contour.points.reverse();
                }
            } else if contour.signed_area() < 0.0 {
                contour.points.reverse();
            }
            contours.push(contour);
        }
    }

    contours
}

/// Аппроксимирует замкнутый полигон последовательностью квадратичных кривых.
pub fn fit_polygon(points: &[Point], settings: &OutlineSettings) -> Vec<ContourPoint> {
    let points = dedup(points, 0.01);
    if points.len() < 3 {
        return Vec::new();
    }

    let chains = split_into_chains(&points, settings);
    let mut out: Vec<ContourPoint> = Vec::new();
    for chain in chains {
        if chain.len() < 2 {
            continue;
        }
        let simplified = simplify_rdp(&chain, settings.simplify_epsilon);
        let simplified = if simplified.len() >= 2 {
            simplified
        } else {
            chain.clone()
        };

        // Начало цепочки — точка на контуре. Если цепочка продолжает предыдущую,
        // общая точка уже присутствует в `out`.
        let start = simplified[0];
        let need_start = match out.last() {
            None => true,
            Some(last) => {
                (last.x - start.x).abs() > 0.01 || (last.y - start.y).abs() > 0.01
            }
        };
        if need_start {
            out.push(ContourPoint::on_curve(start.x, start.y));
        }

        fit_chain(&simplified, settings.curve_tolerance, &mut out);
    }

    // Последняя точка не должна совпадать с первой (контур замыкается неявно).
    if out.len() > 1 {
        let first = out[0];
        let last = *out.last().unwrap();
        if first.on_curve
            && last.on_curve
            && (first.x - last.x).abs() < 0.01
            && (first.y - last.y).abs() < 0.01
        {
            out.pop();
        }
    }

    out
}

/// Делит полигон на цепочки, разрывая его в местах углов.
fn split_into_chains(points: &[Point], settings: &OutlineSettings) -> Vec<Vec<Point>> {
    let n = points.len();
    let threshold = settings.corner_angle_degrees.to_radians();

    let mut corners = Vec::with_capacity(n);
    for i in 0..n {
        let prev = points[(i + n - 1) % n];
        let cur = points[i];
        let next = points[(i + 1) % n];
        corners.push(turn_angle(prev, cur, next) < threshold);
    }

    // Точка разрыва: самая «острая» вершина. Для гладкого контура берём первую.
    let mut start = 0usize;
    let mut best = 0.0f32;
    for i in 0..n {
        let prev = points[(i + n - 1) % n];
        let cur = points[i];
        let next = points[(i + 1) % n];
        let angle = turn_angle(prev, cur, next);
        if angle < best {
            best = angle;
            start = i;
        }
    }

    let mut chains: Vec<Vec<Point>> = Vec::new();
    let mut current: Vec<Point> = Vec::with_capacity(n + 1);

    for step in 0..=n {
        let index = (start + step) % n;
        current.push(points[index]);
        let is_corner = corners[index];
        if (is_corner && step > 0) || step == n {
            chains.push(std::mem::take(&mut current));
        }
    }

    chains.retain(|c| c.len() >= 2);
    chains
}

/// Аппроксимирует цепочку точек квадратичными кривыми (алгоритм Шнейдера).
///
/// Первая точка цепочки должна уже присутствовать в `out`.
fn fit_chain(points: &[Point], tolerance: f32, out: &mut Vec<ContourPoint>) {
    if points.len() < 2 {
        return;
    }
    if points.len() == 2 {
        out.push(ContourPoint::on_curve(points[1].x, points[1].y));
        return;
    }

    let (control, errors) = fit_quadratic(points);
    let max_error = errors.iter().copied().fold(0.0f32, f32::max);

    if max_error <= tolerance || points.len() <= 3 {
        out.push(ContourPoint::off_curve(control.x, control.y));
        out.push(ContourPoint::on_curve(points[points.len() - 1].x, points[points.len() - 1].y));
        return;
    }

    let split = errors
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(1)
        .clamp(1, points.len() - 1);

    fit_chain(&points[..=split], tolerance, out);
    fit_chain(&points[split..], tolerance, out);
}

/// Находит управляющую точку квадратичной кривой методом наименьших квадратов.
fn fit_quadratic(points: &[Point]) -> (Point, Vec<f32>) {
    let p0 = points[0];
    let p2 = points[points.len() - 1];

    // Параметризация по длине хорды: params[0] соответствует первой точке.
    let mut params = Vec::with_capacity(points.len());
    let mut lengths = Vec::with_capacity(points.len());
    let mut total = 0.0f32;
    for i in 1..points.len() {
        total += points[i].distance(points[i - 1]);
        lengths.push(total);
    }
    params.push(0.0);
    if total <= f32::EPSILON {
        for _ in 1..points.len() {
            params.push(0.0);
        }
    } else {
        for len in lengths {
            params.push(len / total);
        }
    }

    let mut numerator = Point::ZERO;
    let mut denominator = 0.0f32;
    for (i, &t) in params.iter().enumerate() {
        let u = (1.0 - t) * (1.0 - t);
        let v = 2.0 * t * (1.0 - t);
        let w = t * t;
        let target = points[i] - p0 * u - p2 * w;
        numerator += target * v;
        denominator += v * v;
    }

    let control = if denominator > f32::EPSILON {
        numerator * (1.0 / denominator)
    } else {
        (p0 + p2) * 0.5
    };

    // Погрешность в каждой точке.
    let mut errors = Vec::with_capacity(points.len());
    for (i, &t) in params.iter().enumerate() {
        let u = (1.0 - t) * (1.0 - t);
        let v = 2.0 * t * (1.0 - t);
        let w = t * t;
        let curve_point = p0 * u + control * v + p2 * w;
        errors.push(curve_point.distance(points[i]));
    }

    (control, errors)
}

/// Ориентированная площадь контура (по управляющим точкам).
pub fn contour_area(points: &[ContourPoint]) -> f32 {
    let poly: Vec<Point> = points.iter().map(|p| Point::new(p.x, p.y)).collect();
    polygon_signed_area(&poly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brush::{BrushKind, BrushSettings};
    use crate::model::{Stroke, StrokePoint};

    fn rectangle_stroke() -> Stroke {
        let mut stroke = Stroke::new(1, BrushKind::Marker, BrushSettings::for_kind(BrushKind::Marker));
        stroke.closed = true;
        stroke.points = vec![
            StrokePoint::new(0.0, 0.0, 0.8),
            StrokePoint::new(200.0, 0.0, 0.8),
            StrokePoint::new(200.0, 200.0, 0.8),
            StrokePoint::new(0.0, 200.0, 0.8),
        ];
        stroke
    }

    #[test]
    fn closed_stroke_yields_two_contours() {
        let glyph = Glyph::new(0x0041, "A", 600);
        let mut glyph = glyph;
        glyph.strokes.push(rectangle_stroke());
        let contours = build_glyph_outline(&glyph, &OutlineSettings::default());
        assert_eq!(contours.len(), 2);
        assert!(contours[0].signed_area() > 0.0);
        assert!(contours[1].signed_area() < 0.0);
        assert!(contours[0].points.len() >= 4);
    }

    #[test]
    fn fit_respects_tolerance() {
        let mut points = Vec::new();
        for i in 0..=32 {
            let t = i as f32 / 32.0;
            points.push(Point::new(t * 100.0, (t * std::f32::consts::TAU).sin() * 20.0));
        }
        let fitted = fit_polygon(&points, &OutlineSettings::default());
        assert!(fitted.len() >= 4);
        assert!(fitted.iter().any(|p| !p.on_curve));
    }
}
