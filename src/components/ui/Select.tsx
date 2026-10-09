import { ChevronDown } from "lucide-react";

interface SelectProps {
  value: string;
  onChange: (value: string) => void;
  options: { value: string; label: string }[];
  disabled?: boolean;
  placeholder?: string;
}

export function Select({ value, onChange, options, disabled, placeholder }: SelectProps) {
  return (
    <div className="relative">
      <select
        value={value}
        disabled={disabled}
        onChange={(e) => onChange(e.target.value)}
        className="w-full appearance-none rounded-lg bg-(--color-surface) border border-(--color-border) px-3 py-2 pr-9 text-sm text-(--color-text) focus:border-(--color-accent) outline-none disabled:opacity-40 disabled:cursor-not-allowed"
      >
        {placeholder && (
          <option value="" disabled>
            {placeholder}
          </option>
        )}
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
      <ChevronDown className="w-4 h-4 text-(--color-text-dim) absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none" />
    </div>
  );
}
