import { useState, type ReactNode } from "react";
import { HelpCircle } from "lucide-react";

export function Tooltip({ text, children }: { text: string; children?: ReactNode }) {
  const [open, setOpen] = useState(false);
  return (
    <span className="relative inline-flex items-center">
      <button
        type="button"
        className="text-(--color-text-dim) hover:text-(--color-text-muted) transition-colors"
        onMouseEnter={() => setOpen(true)}
        onMouseLeave={() => setOpen(false)}
        onFocus={() => setOpen(true)}
        onBlur={() => setOpen(false)}
        aria-label={text}
      >
        {children ?? <HelpCircle className="w-3.5 h-3.5" />}
      </button>
      {open && (
        <span
          role="tooltip"
          className="absolute z-20 bottom-full left-1/2 -translate-x-1/2 mb-2 w-56 rounded-lg border border-(--color-border) bg-(--color-panel) px-3 py-2 text-xs text-(--color-text-muted) shadow-xl shadow-black/40"
        >
          {text}
        </span>
      )}
    </span>
  );
}
