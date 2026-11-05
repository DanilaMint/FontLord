//! Геометрические примитивы: точки, полилинии, сглаживание и упрощение.

use serde::{Deserialize, Serialize};

/// Точка в системе координат шрифта (ось Y направлена вверх).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    /// Координата X.
    pub x: f32,
    /// Координата Y.
    pub y: f32,
}

impl Point {
    /// Нулевая точка.
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    /// Создаёт точку.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Линейная интерполяция.
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    /// Расстояние до другой точки.
    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    /// Длина вектора.
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Квадрат длины вектора.
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Нормализованный вектор (нулевой вектор остаётся нулевым).
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len <= f32::EPSILON {
            Self::ZERO
        } else {
            Self::new(self.x / len, self.y / len)
        }
    }

    /// Вектор, перпендикулярный данному (поворот на +90°).
    pub fn perpendicular(self) -> Self {
        Self::new(-self.y, self.x)
    }

    /// Скалярное произведение.
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Угол вектора в радианах.
    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }

    /// Ориентированная площадь треугольника `(self, a, b)`.
    pub fn cross(self, a: Self, b: Self) -> f32 {
        (a.x - self.x) * (b.y - self.y) - (a.y - self.y) * (b.x - self.x)
    }
}

impl std::ops::Add for Point {
    type Output = Point;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Point {
    type Output = Point;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Neg for Point {
    type Output = Point;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl std::ops::Mul<f32> for Point {
    type Output = Point;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl std::ops::AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

/// Границы набора точек.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BBox {
    /// Минимальная X.
    pub x_min: f32,
    /// Минимальная Y.
    pub y_min: f32,
    /// Максимальная X.
    pub x_max: f32,
    /// Максимальная Y.
    pub y_max: f32,
}

impl BBox {
    /// Пустые границы.
    pub const EMPTY: BBox = BBox {
        x_min: f32::INFINITY,
        y_min: f32::INFINITY,
        x_max: f32::NEG_INFINITY,
        y_max: f32::NEG_INFINITY,
    };

    /// Расширяет границы точкой.
    pub fn extend(&mut self, p: Point) {
        self.x_min = self.x_min.min(p.x);
        self.y_min = self.y_min.min(p.y);
        self.x_max = self.x_max.max(p.x);
        self.y_max = self.y_max.max(p.y);
    }

    /// Ширина границ.
    pub fn width(&self) -> f32 {
        self.x_max - self.x_min
    }

    /// Высота границ.
    pub fn height(&self) -> f32 {
        self.y_max - self.y_min
    }

    /// Признак пустых границ.
    pub fn is_empty(&self) -> bool {
        self.x_min > self.x_max || self.y_min > self.y_max
    }
}

/// Вычисляет границы набора точек.
pub fn bounding_box(points: &[Point]) -> BBox {
    let mut bbox = BBox::EMPTY;
    for &p in points {
        bbox.extend(p);
    }
    bbox
}

/// Длина полилинии.
pub fn polyline_length(points: &[Point]) -> f32 {
    points.windows(2).map(|w| w[1].distance(w[0])).sum::<f32>()
}

/// Удаляет подряд идущие совпадающие точки.
pub fn dedup(points: &[Point], epsilon: f32) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::with_capacity(points.len());
    for &p in points {
        if let Some(last) = out.last() {
            if last.distance(p) <= epsilon {
                continue;
            }
        }
        out.push(p);
    }
    out
}

/// Пересемплирует полилинию с равномерным шагом по длине дуги.
///
/// Конечные точки сохраняются. Значения `values` (например, нажим) интерполируются
/// линейно по длине дуги.
pub fn resample(points: &[Point], values: &[f32], step: f32) -> (Vec<Point>, Vec<f32>) {
    if points.len() < 2 {
        return (points.to_vec(), values.to_vec());
    }
    let step = step.max(1e-3);

    let mut cumulative = Vec::with_capacity(points.len());
    cumulative.push(0.0);
    for i in 1..points.len() {
        cumulative.push(cumulative[i - 1] + points[i].distance(points[i - 1]));
    }
    let total = cumulative[cumulative.len() - 1];
    if total <= step {
        return (points.to_vec(), values.to_vec());
    }

    let count = (total / step).floor() as usize;
    let mut out_points = Vec::with_capacity(count + 1);
    let mut out_values = Vec::with_capacity(count + 1);
    out_points.push(points[0]);
    out_values.push(values.first().copied().unwrap_or(0.5));

    let mut segment = 0usize;
    for i in 1..=count {
        let target = i as f32 * step;
        while segment + 1 < cumulative.len() && cumulative[segment + 1] < target {
            segment += 1;
        }
        let seg_start = cumulative[segment];
        let seg_end = cumulative[segment + 1];
        let t = if (seg_end - seg_start).abs() < f32::EPSILON {
            0.0
        } else {
            (target - seg_start) / (seg_end - seg_start)
        };
        out_points.push(points[segment].lerp(points[segment + 1], t));
        let v0 = values.get(segment).copied().unwrap_or(0.5);
        let v1 = values.get(segment + 1).copied().unwrap_or(v0);
        out_values.push(v0 + (v1 - v0) * t);
    }

    let last = *points.last().unwrap();
    if out_points
        .last()
        .map(|p| p.distance(last) > step * 0.25)
        .unwrap_or(true)
    {
        out_points.push(last);
        out_values.push(values.last().copied().unwrap_or(0.5));
    }

    (out_points, out_values)
}

/// Сглаживание полилинии сплайном Катмулла–Рома.
///
/// `closed` замыкает сплайн в кольцо.
pub fn smooth_catmull_rom(points: &[Point], closed: bool, subdivisions: usize) -> Vec<Point> {
    let n = points.len();
    if n < 3 {
        return points.to_vec();
    }
    let subdivisions = subdivisions.max(1);
    let mut out = Vec::with_capacity(n * subdivisions + 1);

    let index = |i: isize| -> Point {
        if closed {
            let len = n as isize;
            let wrapped = ((i % len) + len) % len;
            points[wrapped as usize]
        } else {
            points[i.clamp(0, n as isize - 1) as usize]
        }
    };

    let segments = if closed { n } else { n - 1 };
    for i in 0..segments {
        let p0 = index(i as isize - 1);
        let p1 = index(i as isize);
        let p2 = index(i as isize + 1);
        let p3 = index(i as isize + 2);
        for s in 0..subdivisions {
            let t = s as f32 / subdivisions as f32;
            let t2 = t * t;
            let t3 = t2 * t;
            let point = p0 * (-0.5 * t3 + t2 - 0.5 * t)
                + p1 * (1.5 * t3 - 2.5 * t2 + 1.0)
                + p2 * (-1.5 * t3 + 2.0 * t2 + 0.5 * t)
                + p3 * (0.5 * t3 - 0.5 * t2);
            out.push(point);
        }
    }
    if !closed {
        out.push(*points.last().unwrap());
    }
    out
}

/// Упрощение полилинии алгоритмом Рассела–Дойла (RDP).
pub fn simplify_rdp(points: &[Point], epsilon: f32) -> Vec<Point> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;

    let mut stack = vec![(0usize, points.len() - 1)];
    while let Some((start, end)) = stack.pop() {
        let mut max_dist = 0.0;
        let mut index = start;
        for i in (start + 1)..end {
            let dist = perpendicular_distance(points[i], points[start], points[end]);
            if dist > max_dist {
                max_dist = dist;
                index = i;
            }
        }
        if max_dist > epsilon {
            keep[index] = true;
            stack.push((start, index));
            stack.push((index, end));
        }
    }

    points
        .iter()
        .zip(keep)
        .filter_map(|(p, k)| if k { Some(*p) } else { None })
        .collect()
}

/// Расстояние от точки до отрезка.
pub fn perpendicular_distance(p: Point, a: Point, b: Point) -> f32 {
    let ab = b - a;
    let len2 = ab.length_squared();
    if len2 < f32::EPSILON {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

/// Угол между векторами в точке (внутренний угол полилинии), в радианах.
pub fn turn_angle(prev: Point, cur: Point, next: Point) -> f32 {
    let a = (prev - cur).normalized();
    let b = (next - cur).normalized();
    if a.length_squared() < f32::EPSILON || b.length_squared() < f32::EPSILON {
        return 0.0;
    }
    let cos = a.dot(b).clamp(-1.0, 1.0);
    cos.acos()
}

/// Ориентированная площадь замкнутого многоугольника.
pub fn polygon_signed_area(points: &[Point]) -> f32 {
    if points.len() < 3 {
        return 0.0;
    }
    let mut sum = 0.0f64;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        sum += (a.x as f64) * (b.y as f64) - (b.x as f64) * (a.y as f64);
    }
    (sum * 0.5) as f32
}

/// Разворачивает многоугольник, если его ориентированная площадь отрицательна.
pub fn ensure_ccw(points: &mut [Point]) {
    if polygon_signed_area(points) < 0.0 {
        points.reverse();
    }
}

/// Разворачивает многоугольник, если его ориентированная площадь положительна.
pub fn ensure_cw(points: &mut [Point]) {
    if polygon_signed_area(points) > 0.0 {
        points.reverse();
    }
}

/// Плавная ступенька на отрезке `0..1`.
pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Устраняет шум нажатия, оставляя среднее скользящее окно.
pub fn smooth_pressures(values: &[f32], window: usize) -> Vec<f32> {
    if window <= 1 || values.len() < 3 {
        return values.to_vec();
    }
    let half = window / 2;
    let n = values.len();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let start = i.saturating_sub(half);
        let end = (i + half + 1).min(n);
        let sum: f32 = values[start..end].iter().sum();
        out.push(sum / (end - start) as f32);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_preserves_endpoints() {
        let points = vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0)];
        let values = vec![0.5, 0.5];
        let (pts, vals) = resample(&points, &values, 10.0);
        assert_eq!(pts.len(), 11);
        assert_eq!(pts[0], Point::new(0.0, 0.0));
        assert_eq!(pts[10], Point::new(100.0, 0.0));
        assert_eq!(vals.len(), 11);
    }

    #[test]
    fn rdp_keeps_corners() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 50.0),
            Point::new(100.0, 0.0),
        ];
        let simplified = simplify_rdp(&points, 1.0);
        assert_eq!(simplified.len(), 3);
    }

    #[test]
    fn signed_area_orientation() {
        let ccw = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ];
        assert!(polygon_signed_area(&ccw) > 0.0);
        let mut cw = ccw.clone();
        cw.reverse();
        assert!(polygon_signed_area(&cw) < 0.0);
    }
}
