/**
 * Панель инструментов кистей: выбор кисти и её параметров.
 */

import { useStore } from '../store';
import type { BrushKind } from '../types';

const KIND_LABELS: Record<BrushKind, string> = {
  pen: 'Перо',
  marker: 'Маркер',
  broadNib: 'Каллиграфия',
  soft: 'Мягкая',
  eraser: 'Ластик',
};

export function BrushPanel() {
  const presets = useStore((state) => state.presets);
  const brushKind = useStore((state) => state.brushKind);
  const brushSettings = useStore((state) => state.brushSettings);
  const setBrushKind = useStore((state) => state.setBrushKind);
  const setBrushSettings = useStore((state) => state.setBrushSettings);
  const applyPreset = useStore((state) => state.applyPreset);

  const angleDegrees = Math.round((brushSettings.angle * 180) / Math.PI);

  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Кисти</h2>
      </div>

      <div className="brush-kinds">
        {(Object.keys(KIND_LABELS) as BrushKind[]).map((kind) => (
          <button
            type="button"
            key={kind}
            className={`chip${brushKind === kind ? ' active' : ''}`}
            onClick={() => setBrushKind(kind)}
          >
            {KIND_LABELS[kind]}
          </button>
        ))}
      </div>

      <div className="presets">
        {presets.map((preset) => (
          <button
            type="button"
            key={preset.id}
            className="preset"
            title={preset.description}
            onClick={() => applyPreset(preset)}
          >
            {preset.name}
          </button>
        ))}
      </div>

      <div className="sliders">
        <Slider
          label="Толщина"
          value={brushSettings.width}
          min={5}
          max={400}
          step={1}
          suffix=" ед."
          onChange={(value) => setBrushSettings({ width: value })}
        />
        {brushKind === 'broadNib' && (
          <Slider
            label="Угол пера"
            value={angleDegrees}
            min={0}
            max={180}
            step={1}
            suffix="°"
            onChange={(value) => setBrushSettings({ angle: (value * Math.PI) / 180 })}
          />
        )}
        {brushKind !== 'broadNib' && brushKind !== 'eraser' && (
          <>
            <Slider
              label="Мин. толщина"
              value={brushSettings.minWidthRatio}
              min={0}
              max={1}
              step={0.05}
              onChange={(value) => setBrushSettings({ minWidthRatio: value })}
            />
            <Slider
              label="Отклик на нажим"
              value={brushSettings.pressureGamma}
              min={0.2}
              max={3}
              step={0.1}
              onChange={(value) => setBrushSettings({ pressureGamma: value })}
            />
          </>
        )}
        {brushKind !== 'eraser' && brushKind !== 'marker' && (
          <>
            <Slider
              label="Сужение в начале"
              value={brushSettings.taperIn}
              min={0}
              max={0.5}
              step={0.01}
              onChange={(value) => setBrushSettings({ taperIn: value })}
            />
            <Slider
              label="Сужение в конце"
              value={brushSettings.taperOut}
              min={0}
              max={0.5}
              step={0.01}
              onChange={(value) => setBrushSettings({ taperOut: value })}
            />
          </>
        )}
      </div>
    </section>
  );
}

interface SliderProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  suffix?: string;
  onChange: (value: number) => void;
}

function Slider({ label, value, min, max, step, suffix = '', onChange }: SliderProps) {
  return (
    <label className="slider">
      <span className="slider-label">
        {label}
        <em>
          {step < 1 ? value.toFixed(2) : Math.round(value)}
          {suffix}
        </em>
      </span>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
      />
    </label>
  );
}
