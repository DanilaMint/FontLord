/**
 * Преобразования между системами координат шрифта и холста.
 *
 * В шрифте ось Y направлена вверх, на холсте — вниз.
 */

/** Параметры отображения холста. */
export interface Viewport {
  /** Масштаб (пикселей на шрифтовую единицу). */
  pixelsPerUnit: number;
  /** Позиция начала координат по X (пиксели). */
  originX: number;
  /** Позиция базовой линии по Y (пиксели). */
  originY: number;
}

/** Переводит точку холста в шрифтовые единицы. */
export function canvasToFont(
  x: number,
  y: number,
  viewport: Viewport,
): { x: number; y: number } {
  return {
    x: (x - viewport.originX) / viewport.pixelsPerUnit,
    y: (viewport.originY - y) / viewport.pixelsPerUnit,
  };
}

/** Переводит шрифтовые единицы в координаты холста. */
export function fontToCanvas(
  x: number,
  y: number,
  viewport: Viewport,
): { x: number; y: number } {
  return {
    x: viewport.originX + x * viewport.pixelsPerUnit,
    y: viewport.originY - y * viewport.pixelsPerUnit,
  };
}

/**
 * Строит `Path2D` по контуру глифа.
 *
 * Поддерживаются правила TrueType: подразумеваемые точки на кривой между двумя
 * управляющими точками и замыкание контура неявной точкой.
 */
export function contourToPath(contour: { points: { x: number; y: number; onCurve: boolean }[] }): Path2D {
  const path = new Path2D();
  const points = contour.points;
  if (points.length === 0) return path;

  const onCurveIndexes = points
    .map((point, index) => (point.onCurve ? index : -1))
    .filter((index) => index >= 0);

  let ordered = points;
  if (onCurveIndexes.length === 0) {
    // Все точки управляющие: начинаем с подразумеваемой середины.
    const first = midpoint(points[points.length - 1], points[0]);
    path.moveTo(first.x, first.y);
    ordered = [...points, { x: first.x, y: first.y, onCurve: true }];
  } else {
    const start = onCurveIndexes[0];
    ordered = [...points.slice(start), ...points.slice(0, start)];
    path.moveTo(ordered[0].x, ordered[0].y);
  }

  let index = 1;
  const count = ordered.length;
  while (index <= count) {
    const current = ordered[index % count];
    if (current.onCurve) {
      path.lineTo(current.x, current.y);
      index += 1;
      continue;
    }

    const next = ordered[(index + 1) % count];
    if (next && next.onCurve) {
      path.quadraticCurveTo(current.x, current.y, next.x, next.y);
      index += 2;
    } else {
      // Две управляющие точки подряд: подразумеваемая точка на кривой.
      const implied = midpoint(current, next);
      path.quadraticCurveTo(current.x, current.y, implied.x, implied.y);
      index += 1;
    }
  }

  path.closePath();
  return path;
}

/** Середина между двумя точками. */
function midpoint(a: { x: number; y: number }, b: { x: number; y: number }) {
  return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
}
