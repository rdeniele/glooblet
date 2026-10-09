interface NumberInputProps {
  value: number | null;
  onChange: (value: number | null) => void;
  min?: number;
  max?: number;
  placeholder?: string;
  suffix?: string;
  disabled?: boolean;
}

export function NumberInput({ value, onChange, min, max, placeholder, suffix, disabled }: NumberInputProps) {
  return (
    <div className="relative">
      <input
        type="number"
        inputMode="decimal"
        value={value ?? ""}
        min={min}
        max={max}
        disabled={disabled}
        placeholder={placeholder}
        onChange={(e) => {
          const raw = e.target.value;
          onChange(raw === "" ? null : Number(raw));
        }}
        className="w-full rounded-lg bg-(--color-surface) border border-(--color-border) px-3 py-2 text-sm text-(--color-text) placeholder:text-(--color-text-dim) focus:border-(--color-accent) outline-none disabled:opacity-40 disabled:cursor-not-allowed"
      />
      {suffix && (
        <span className="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-(--color-text-dim)">
          {suffix}
        </span>
      )}
    </div>
  );
}
