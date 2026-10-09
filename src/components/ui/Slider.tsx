interface SliderProps {
  value: number;
  min: number;
  max: number;
  step?: number;
  onChange: (value: number) => void;
  onCommit?: (value: number) => void;
  disabled?: boolean;
  formatValue?: (value: number) => string;
  label?: string;
}

export function Slider({
  value,
  min,
  max,
  step = 1,
  onChange,
  onCommit,
  disabled,
  formatValue,
  label,
}: SliderProps) {
  const pct = ((value - min) / (max - min)) * 100;
  return (
    <div className="flex items-center gap-3 w-full">
      <input
        type="range"
        aria-label={label}
        min={min}
        max={max}
        step={step}
        value={value}
        disabled={disabled}
        onChange={(e) => onChange(Number(e.target.value))}
        onPointerUp={(e) => onCommit?.(Number((e.target as HTMLInputElement).value))}
        onKeyUp={(e) => onCommit?.(Number((e.target as HTMLInputElement).value))}
        className="flex-1 h-1.5 rounded-full appearance-none cursor-pointer disabled:cursor-not-allowed disabled:opacity-40 accent-(--color-accent)"
        style={{
          background: `linear-gradient(to right, var(--color-accent) ${pct}%, var(--color-surface) ${pct}%)`,
        }}
      />
      <span className="text-sm font-medium text-(--color-text) w-12 text-right tabular-nums">
        {formatValue ? formatValue(value) : value}
      </span>
    </div>
  );
}
