/**
 * Холст редактора глифа: сетка, направляющие, контуры и рисование кистью.
 *
 * В режиме курсора холст работает как редактор выделения: можно обвести
 * область, выбрать штрихи, перетащить их, а также удалить, скопировать,
 * вырезать и вставить.
 */

import { useCallback, useEffect, useRef, useState } from 'react';

import { canvasToFont, contourToPath, fontToCanvas, type Viewport } from '../canvas';
import { useT } from '../i18n';
import { useStore } from '../store';
import { CANVAS_PALETTES, type CanvasPalette } from '../theme';
import type { BrushKind, Stroke, StrokePoint } from '../types';

/** Нажам мыши: у мыши нет датчика давления, поэтому используем фиксированное значение. */
const MOUSE_PRESSURE = 0.75;

/** Радиус попадания по штриху (в шрифтовых единицах). */
const HIT_RADIUS = 14;

interface Props {
  /** Признак активности холста (внутри редактора). */
  active: boolean;
}

/** Прямоугольник выделения в шрифтовых единицах. */
interface Rect {
  x0: number;
  y0: number;
  x1: number;
  y1: number;
}

export function GlyphCanvas({ active }: Props) {
  const t = useT();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [drawing, setDrawing] = useState(false);
  const [livePoints, setLivePoints] = useState<StrokePoint[]>([]);
  const [closedMode, setClosedMode] = useState(false);
  /** Прямоугольник выделения (режим курсора). */
  const [band, setBand] = useState<Rect | null>(null);
  /** Смещение перетаскиваемых штрихов (режим курсора). */
  const [dragDelta, setDragDelta] = useState<{ x: number; y: number } | null>(null);

  const glyph = useStore((state) => state.glyph);
  const meta = useStore((state) => state.meta);
  const brushKind = useStore((state) => state.brushKind);
  const brushSettings = useStore((state) => state.brushSettings);
  const nextStrokeId = useStore((state) => state.nextStrokeId);
  const theme = useStore((state) => state.theme);
  const zoom = useStore((state) => state.zoom);
  const zoomIn = useStore((state) => state.zoomIn);
  const zoomOut = useStore((state) => state.zoomOut);
  const zoomReset = useStore((state) => state.zoomReset);
  const applyStroke = useStore((state) => state.applyStroke);
  const eraseAt = useStore((state) => state.eraseAt);
  const cursorMode = useStore((state) => state.cursorMode);
  const selectedStrokeIds = useStore((state) => state.selectedStrokeIds);
  const setSelection = useStore((state) => state.setSelection);
  const toggleStrokeSelection = useStore((state) => state.toggleStrokeSelection);
  const clearSelection = useStore((state) => state.clearSelection);
  const moveSelectedStrokes = useStore((state) => state.moveSelectedStrokes);

  const palette = CANVAS_PALETTES[theme];
  const unitsPerEm = meta?.unitsPerEm ?? 1000;
  const ascender = meta?.ascender ?? 800;
  const descender = meta?.descender ?? -200;
  const xHeight = meta?.xHeight ?? 500;
  const capHeight = meta?.capHeight ?? 700;

  /** Пересчитывает геометрию холста и перерисовывает сцену. */
  const redraw = useCallback(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;

    const dpr = window.devicePixelRatio || 1;
    const width = container.clientWidth;
    const height = container.clientHeight;
    canvas.width = Math.max(1, Math.floor(width * dpr));
    canvas.height = Math.max(1, Math.floor(height * dpr));
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;

    const context = canvas.getContext('2d');
    if (!context) return;
    context.setTransform(dpr, 0, 0, dpr, 0, 0);
    context.clearRect(0, 0, width, height);

    // Масштаб: квадрат em помещается в холст с полями.
    const margin = 48;
    const fit = Math.min(width - margin * 2, height - margin * 2) / unitsPerEm;
    const pixelsPerUnit = fit * zoom;
    const originX = (width - unitsPerEm * pixelsPerUnit) / 2;
    const originY = (height + (ascender + descender) * pixelsPerUnit) / 2;
    const viewport: Viewport = { pixelsPerUnit, originX, originY };
    viewportRef.current = viewport;

    // Фон.
    context.fillStyle = palette.background;
    context.fillRect(0, 0, width, height);

    // Квадрат em.
    const emTop = fontToCanvas(0, ascender, viewport).y;
    const emHeight = (ascender - descender) * pixelsPerUnit;
    context.strokeStyle = palette.emSquare;
    context.lineWidth = 1;
    context.strokeRect(originX, emTop, unitsPerEm * pixelsPerUnit, emHeight);

    // Направляющие.
    const guides: { y: number; color: string; label: string }[] = [
      { y: ascender, color: palette.guide, label: 'ascender' },
      { y: capHeight, color: palette.guide, label: 'cap' },
      { y: xHeight, color: palette.guide, label: 'x-height' },
      { y: 0, color: palette.baseline, label: 'baseline' },
      { y: descender, color: palette.guide, label: 'descender' },
    ];
    context.font = '10px "JetBrains Mono", monospace';
    for (const guide of guides) {
      const y = Math.round(fontToCanvas(0, guide.y, viewport).y) + 0.5;
      context.strokeStyle = guide.color;
      context.beginPath();
      context.moveTo(originX, y);
      context.lineTo(originX + unitsPerEm * pixelsPerUnit, y);
      context.stroke();
      context.fillStyle = guide.color;
      context.fillText(guide.label, originX + unitsPerEm * pixelsPerUnit + 6, y + 3);
    }

    // Сетка внутри em.
    context.strokeStyle = palette.grid;
    for (let x = 0; x <= unitsPerEm; x += 100) {
      const px = Math.round(fontToCanvas(x, 0, viewport).x) + 0.5;
      context.beginPath();
      context.moveTo(px, emTop);
      context.lineTo(px, emTop + emHeight);
      context.stroke();
    }

    // Ширина продвижения.
    const advance = glyph?.advanceWidth ?? unitsPerEm / 2;
    const advanceWidth = advance * pixelsPerUnit;
    context.fillStyle = palette.advanceFill;
    context.fillRect(originX, emTop, advanceWidth, emHeight);
    context.strokeStyle = palette.advanceStroke;
    context.setLineDash([4, 4]);
    context.strokeRect(originX, emTop, advanceWidth, emHeight);
    context.setLineDash([]);

    // Контуры глифа: путь строится в шрифтовых единицах, а трансформация
    // переводит его в координаты холста (с перевёрнутой осью Y).
    if (glyph) {
      context.save();
      context.translate(viewport.originX, viewport.originY);
      context.scale(pixelsPerUnit, -pixelsPerUnit);
      context.fillStyle = palette.glyphFill;
      context.strokeStyle = palette.glyphStroke;
      context.lineWidth = 1.2 / pixelsPerUnit;
      for (const contour of glyph.contours) {
        const path = contourToPath(contour);
        context.fill(path, 'nonzero');
        context.stroke(path);
      }
      context.restore();
    }

    // Выделенные штрихи (и их перетаскивание).
    if (glyph && selectedStrokeIds.length > 0) {
      context.save();
      context.translate(viewport.originX, viewport.originY);
      context.scale(pixelsPerUnit, -pixelsPerUnit);
      context.strokeStyle = palette.selection;
      context.fillStyle = palette.selection;
      context.lineCap = 'round';
      context.lineJoin = 'round';
      context.globalAlpha = 0.85;
      for (const stroke of glyph.strokes) {
        if (!selectedStrokeIds.includes(stroke.id)) continue;
        drawStrokeHighlight(context, stroke, dragDelta);
      }
      context.restore();
    }

    // Прямоугольник выделения.
    if (band) {
      const left = fontToCanvas(band.x0, band.y1, viewport).x;
      const right = fontToCanvas(band.x1, band.y0, viewport).x;
      const top = fontToCanvas(0, band.y1, viewport).y;
      const bottom = fontToCanvas(0, band.y0, viewport).y;
      context.fillStyle = 'rgba(120, 190, 255, 0.12)';
      context.fillRect(left, top, right - left, bottom - top);
      context.strokeStyle = palette.selection;
      context.setLineDash([5, 4]);
      context.lineWidth = 1;
      context.strokeRect(left, top, right - left, bottom - top);
      context.setLineDash([]);
    }

    // Текущий штрих (предпросмотр во время рисования).
    if (livePoints.length > 1) {
      drawStrokePreview(context, viewport, livePoints, brushKind, brushSettings.width, palette);
    }
  }, [
    ascender,
    brushKind,
    brushSettings.width,
    capHeight,
    descender,
    dragDelta,
    glyph,
    livePoints,
    palette,
    selectedStrokeIds,
    unitsPerEm,
    xHeight,
    zoom,
  ]);

  const viewportRef = useRef<Viewport>({ pixelsPerUnit: 1, originX: 0, originY: 0 });
  /** Точки текущего штриха ( синхронно с событиями указателя). */
  const pointsRef = useRef<StrokePoint[]>([]);
  /** Начало текущего штриха — от него считается выравнивание по Shift. */
  const strokeAnchorRef = useRef<{ x: number; y: number }>({ x: 0, y: 0 });
  /** Текущее взаимодействие в режиме курсора. */
  const interactionRef = useRef<
    { kind: 'band' | 'drag'; start: { x: number; y: number } } | null
  >(null);
  /** Прямоугольник выделения (актуальное значение). */
  const bandRef = useRef<Rect | null>(null);

  useEffect(() => {
    redraw();
  }, [redraw]);

  useEffect(() => {
    const observer = new ResizeObserver(() => redraw());
    if (containerRef.current) observer.observe(containerRef.current);
    return () => observer.disconnect();
  }, [redraw]);

  /** Преобразование координат указателя в шрифтовые единицы. */
  const toFont = (event: React.PointerEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return { x: 0, y: 0 };
    const rect = canvas.getBoundingClientRect();
    return canvasToFont(event.clientX - rect.left, event.clientY - rect.top, viewportRef.current);
  };

  /** Ищет штрих под указателем. */
  const hitStroke = (point: { x: number; y: number }): number | null => {
    if (!glyph) return null;
    let best: number | null = null;
    let bestDistance = HIT_RADIUS;
    for (const stroke of glyph.strokes) {
      for (const node of stroke.points) {
        const distance = Math.hypot(node.x - point.x, node.y - point.y);
        if (distance < bestDistance) {
          bestDistance = distance;
          best = stroke.id;
        }
      }
    }
    return best;
  };

  /** Штрихи, попавшие в прямоугольник выделения. */
  const strokesInRect = (rect: Rect): number[] => {
    if (!glyph) return [];
    const x0 = Math.min(rect.x0, rect.x1);
    const x1 = Math.max(rect.x0, rect.x1);
    const y0 = Math.min(rect.y0, rect.y1);
    const y1 = Math.max(rect.y0, rect.y1);
    const ids: number[] = [];
    for (const stroke of glyph.strokes) {
      const inside = stroke.points.some(
        (node) => node.x >= x0 && node.x <= x1 && node.y >= y0 && node.y <= y1,
      );
      if (inside) ids.push(stroke.id);
    }
    return ids;
  };

  const handlePointerDown = (event: React.PointerEvent<HTMLCanvasElement>) => {
    // Глифа может ещё не существовать в проекте: первый штрих его создаст.
    if (!active) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    const point = toFont(event);

    // Режим курсора: выделение и перетаскивание штрихов.
    if (cursorMode) {
      const hit = hitStroke(point);
      if (hit !== null) {
        if (event.shiftKey) {
          toggleStrokeSelection(hit);
        } else if (!selectedStrokeIds.includes(hit)) {
          setSelection([hit]);
        }
        interactionRef.current = { kind: 'drag', start: point };
        setDragDelta({ x: 0, y: 0 });
        return;
      }
      if (!event.shiftKey) clearSelection();
      interactionRef.current = { kind: 'band', start: point };
      bandRef.current = { x0: point.x, y0: point.y, x1: point.x, y1: point.y };
      setBand(bandRef.current);
      return;
    }

    const pressure = pressureOf(event);
    strokeAnchorRef.current = point;
    pointsRef.current = [{ x: point.x, y: point.y, pressure }];
    setDrawing(true);
    setLivePoints([{ x: point.x, y: point.y, pressure }]);
  };

  const handlePointerMove = (event: React.PointerEvent<HTMLCanvasElement>) => {
    if (cursorMode) {
      const interaction = interactionRef.current;
      if (!interaction) return;
      const point = toFont(event);
      const start = interaction.start;
      if (interaction.kind === 'band') {
        bandRef.current = { x0: start.x, y0: start.y, x1: point.x, y1: point.y };
        setBand(bandRef.current);
      } else if (selectedStrokeIds.length > 0) {
        setDragDelta({ x: point.x - start.x, y: point.y - start.y });
      }
      return;
    }

    if (!drawing) return;
    let point = toFont(event);
    // Удержание Shift выравнивает штрих по прямой: направление округляется до
    // ближайших 45°, а длина сохраняется.
    if (event.shiftKey) {
      point = constrainToStraight(point, strokeAnchorRef.current);
    }
    const pressure = pressureOf(event);
    const next = { x: point.x, y: point.y, pressure };
    const last = pointsRef.current[pointsRef.current.length - 1];
    if (last && Math.hypot(last.x - point.x, last.y - point.y) < 1.2) return;
    pointsRef.current = [...pointsRef.current, next];
    setLivePoints(pointsRef.current);
  };

  const handlePointerUp = async () => {
    if (cursorMode) {
      const interaction = interactionRef.current;
      interactionRef.current = null;
      if (interaction?.kind === 'band') {
        const rect = bandRef.current;
        setSelection(rect ? strokesInRect(rect) : []);
        bandRef.current = null;
        setBand(null);
      } else if (dragDelta && (dragDelta.x !== 0 || dragDelta.y !== 0)) {
        await moveSelectedStrokes(dragDelta.x, dragDelta.y);
      }
      setDragDelta(null);
      return;
    }

    if (!drawing) return;
    setDrawing(false);
    const points = pointsRef.current;
    pointsRef.current = [];
    setLivePoints([]);
    if (points.length === 0) return;

    if (brushKind === 'eraser') {
      await eraseAt(
        points.map((p) => [p.x, p.y]),
        brushSettings.width / 2,
      );
      return;
    }

    await applyStroke({
      id: nextStrokeId,
      kind: brushKind,
      settings: brushSettings,
      points,
      closed: closedMode,
    });
  };

  const handleWheel = (event: React.WheelEvent<HTMLCanvasElement>) => {
    if (!event.ctrlKey && !event.metaKey) return;
    event.preventDefault();
    if (event.deltaY < 0) zoomIn();
    else zoomOut();
  };

  const toggleCursorMode = useStore((state) => state.toggleCursorMode);

  return (
    <div className="canvas-wrap" ref={containerRef}>
      <canvas
        ref={canvasRef}
        className={`glyph-canvas${active ? ' active' : ''}${cursorMode ? ' cursor-mode' : ''}`}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerCancel={handlePointerUp}
        onWheel={handleWheel}
      />
      <div className="canvas-tools">
        <button type="button" onClick={zoomOut} title="Уменьшить (Ctrl+−)">
          −
        </button>
        <span>{Math.round(zoom * 100)}%</span>
        <button type="button" onClick={zoomIn} title="Увеличить (Ctrl++)">
          +
        </button>
        <button type="button" onClick={zoomReset} title="Сбросить масштаб (Ctrl+0)">
          {t('canvas.zoomReset')}
        </button>
        <button
          type="button"
          className={closedMode ? 'toggle on' : 'toggle'}
          onClick={() => setClosedMode((value) => !value)}
          title="Замкнуть контур (кольцо)"
          disabled={cursorMode}
        >
          {t('canvas.ring')}
        </button>
        <button
          type="button"
          className={cursorMode ? 'toggle on' : 'toggle'}
          onClick={toggleCursorMode}
          title={t('canvas.cursorHint')}
        >
          {t('canvas.cursor')}
        </button>
        <span className="canvas-hint" title={t('canvas.shiftHint')}>
          {t('canvas.shiftHint')}
        </span>
      </div>
    </div>
  );
}

/** Нажим из события указателя. */
function pressureOf(event: React.PointerEvent<HTMLCanvasElement>): number {
  if (event.pointerType === 'mouse') return MOUSE_PRESSURE;
  if (event.pressure > 0 && event.pressure < 1) return event.pressure;
  return MOUSE_PRESSURE;
}

/**
 * Выравнивает точку по прямой относительно начала штриха.
 *
 * Направление округляется до ближайших 45°, поэтому штрих получается ровным,
 * а его длина сохраняется.
 */
function constrainToStraight(
  point: { x: number; y: number },
  anchor: { x: number; y: number },
): { x: number; y: number } {
  const dx = point.x - anchor.x;
  const dy = point.y - anchor.y;
  const length = Math.hypot(dx, dy);
  if (length < 1e-6) return { x: anchor.x, y: anchor.y };
  const step = Math.PI / 4;
  const angle = Math.atan2(dy, dx);
  const snapped = Math.round(angle / step) * step;
  return {
    x: anchor.x + Math.cos(snapped) * length,
    y: anchor.y + Math.sin(snapped) * length,
  };
}

/** Подсвечивает выбранный штрих (с учётом перетаскивания). */
function drawStrokeHighlight(
  context: CanvasRenderingContext2D,
  stroke: Stroke,
  delta: { x: number; y: number } | null,
) {
  if (stroke.points.length === 0) return;
  context.lineWidth = stroke.settings.width;
  context.beginPath();
  stroke.points.forEach((point, index) => {
    const x = point.x + (delta?.x ?? 0);
    const y = point.y + (delta?.y ?? 0);
    if (index === 0) context.moveTo(x, y);
    else context.lineTo(x, y);
  });
  if (stroke.closed) context.closePath();
  context.stroke();
}

/** Предпросмотр штриха во время рисования. */
function drawStrokePreview(
  context: CanvasRenderingContext2D,
  viewport: Viewport,
  points: StrokePoint[],
  kind: BrushKind,
  width: number,
  palette: CanvasPalette,
) {
  context.save();
  context.translate(viewport.originX, viewport.originY);
  context.scale(viewport.pixelsPerUnit, -viewport.pixelsPerUnit);

  context.strokeStyle = palette.preview;
  context.fillStyle = palette.preview;
  context.lineCap = 'round';
  context.lineJoin = 'round';

  if (kind === 'broadNib') {
    // Перо фиксированной ширины: рисуем ленту вдоль пути.
    context.lineWidth = width;
    context.beginPath();
    points.forEach((point, index) => {
      const canvasPoint = fontToCanvas(point.x, point.y, viewport);
      if (index === 0) context.moveTo(canvasPoint.x, canvasPoint.y);
      else context.lineTo(canvasPoint.x, canvasPoint.y);
    });
    context.stroke();
  } else {
    // Переменная толщина: рисуем последовательность кругов.
    points.forEach((point) => {
      const radius = (width / 2) * (0.4 + 0.6 * point.pressure);
      context.beginPath();
      context.arc(point.x, point.y, radius, 0, Math.PI * 2);
      context.fill();
    });
  }

  context.restore();
}
