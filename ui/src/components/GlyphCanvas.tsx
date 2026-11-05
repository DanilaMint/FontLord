/**
 * Холст редактора глифа: сетка, направляющие, контуры и рисование кистью.
 */

import { useCallback, useEffect, useRef, useState } from 'react';

import { canvasToFont, contourToPath, fontToCanvas, type Viewport } from '../canvas';
import { useStore } from '../store';
import type { BrushKind, StrokePoint } from '../types';

/** Нажам мыши: у мыши нет датчика давления, поэтому используем фиксированное значение. */
const MOUSE_PRESSURE = 0.75;

interface Props {
  /** Признак активности холста (внутри редактора). */
  active: boolean;
}

export function GlyphCanvas({ active }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [zoom, setZoom] = useState(1);
  const [drawing, setDrawing] = useState(false);
  const [livePoints, setLivePoints] = useState<StrokePoint[]>([]);
  const [closedMode, setClosedMode] = useState(false);

  const glyph = useStore((state) => state.glyph);
  const meta = useStore((state) => state.meta);
  const brushKind = useStore((state) => state.brushKind);
  const brushSettings = useStore((state) => state.brushSettings);
  const nextStrokeId = useStore((state) => state.nextStrokeId);
  const applyStroke = useStore((state) => state.applyStroke);
  const eraseAt = useStore((state) => state.eraseAt);

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
    context.fillStyle = '#0f1219';
    context.fillRect(0, 0, width, height);

    // Квадрат em.
    const emTop = fontToCanvas(0, ascender, viewport).y;
    const emHeight = (ascender - descender) * pixelsPerUnit;
    context.strokeStyle = '#252b38';
    context.lineWidth = 1;
    context.strokeRect(originX, emTop, unitsPerEm * pixelsPerUnit, emHeight);

    // Направляющие.
    const guides: { y: number; color: string; label: string }[] = [
      { y: ascender, color: '#3b4a63', label: 'ascender' },
      { y: capHeight, color: '#33405a', label: 'cap' },
      { y: xHeight, color: '#33405a', label: 'x-height' },
      { y: 0, color: '#5b7fb0', label: 'baseline' },
      { y: descender, color: '#3b4a63', label: 'descender' },
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
    context.strokeStyle = 'rgba(120, 140, 180, 0.08)';
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
    context.fillStyle = 'rgba(120, 190, 255, 0.06)';
    context.fillRect(originX, emTop, advanceWidth, emHeight);
    context.strokeStyle = 'rgba(120, 190, 255, 0.25)';
    context.setLineDash([4, 4]);
    context.strokeRect(originX, emTop, advanceWidth, emHeight);
    context.setLineDash([]);

    // Контуры глифа: путь строится в шрифтовых единицах, а трансформация
    // переводит его в координаты холста (с перевёрнутой осью Y).
    if (glyph) {
      context.save();
      context.translate(viewport.originX, viewport.originY);
      context.scale(pixelsPerUnit, -pixelsPerUnit);
      context.fillStyle = '#e9eef7';
      context.strokeStyle = '#78beff';
      context.lineWidth = 1.2 / pixelsPerUnit;
      for (const contour of glyph.contours) {
        const path = contourToPath(contour);
        context.fill(path, 'nonzero');
        context.stroke(path);
      }
      context.restore();
    }

    // Текущий штрих (предпросмотр во время рисования).
    if (livePoints.length > 1) {
      drawStrokePreview(context, viewport, livePoints, brushKind, brushSettings.width);
    }
  }, [
    ascender,
    brushKind,
    brushSettings.width,
    capHeight,
    descender,
    glyph,
    livePoints,
    unitsPerEm,
    xHeight,
    zoom,
  ]);

  const viewportRef = useRef<Viewport>({ pixelsPerUnit: 1, originX: 0, originY: 0 });

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

  const handlePointerDown = (event: React.PointerEvent<HTMLCanvasElement>) => {
    // Глифа может ещё не существовать в проекте: первый штрих его создаст.
    if (!active) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    const point = toFont(event);
    const pressure = pressureOf(event);
    setDrawing(true);
    setLivePoints([{ x: point.x, y: point.y, pressure }]);
  };

  const handlePointerMove = (event: React.PointerEvent<HTMLCanvasElement>) => {
    if (!drawing) return;
    const point = toFont(event);
    const pressure = pressureOf(event);
    setLivePoints((points) => {
      const last = points[points.length - 1];
      if (last && Math.hypot(last.x - point.x, last.y - point.y) < 1.2) return points;
      return [...points, { x: point.x, y: point.y, pressure }];
    });
  };

  const handlePointerUp = async () => {
    if (!drawing) return;
    setDrawing(false);
    const points = livePoints;
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
    setZoom((value) => {
      const next = value * (event.deltaY < 0 ? 1.1 : 1 / 1.1);
      return Math.min(8, Math.max(0.2, next));
    });
  };

  return (
    <div className="canvas-wrap" ref={containerRef}>
      <canvas
        ref={canvasRef}
        className={`glyph-canvas${active ? ' active' : ''}`}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerCancel={handlePointerUp}
        onWheel={handleWheel}
      />
      <div className="canvas-tools">
        <button type="button" onClick={() => setZoom((z) => Math.max(0.2, z / 1.2))}>
          −
        </button>
        <span>{Math.round(zoom * 100)}%</span>
        <button type="button" onClick={() => setZoom((z) => Math.min(8, z * 1.2))}>
          +
        </button>
        <button
          type="button"
          className={closedMode ? 'toggle on' : 'toggle'}
          onClick={() => setClosedMode((value) => !value)}
          title="Замкнуть контур (кольцо)"
        >
          Кольцо
        </button>
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

/** Предпросмотр штриха во время рисования. */
function drawStrokePreview(
  context: CanvasRenderingContext2D,
  viewport: Viewport,
  points: StrokePoint[],
  kind: BrushKind,
  width: number,
) {
  context.save();
  context.translate(viewport.originX, viewport.originY);
  context.scale(viewport.pixelsPerUnit, -viewport.pixelsPerUnit);

  context.strokeStyle = 'rgba(233, 238, 247, 0.85)';
  context.fillStyle = 'rgba(233, 238, 247, 0.85)';
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
