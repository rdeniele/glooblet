import type { ReactNode } from "react";

interface CardProps {
  title?: string;
  description?: string;
  action?: ReactNode;
  children: ReactNode;
  className?: string;
}

export function Card({ title, description, action, children, className = "" }: CardProps) {
  return (
    <div className={`rounded-2xl border border-(--color-border) bg-(--color-panel) p-5 ${className}`}>
      {(title || action) && (
        <div className="flex items-start justify-between gap-3 mb-4">
          <div>
            {title && <h3 className="text-sm font-semibold text-(--color-text)">{title}</h3>}
            {description && <p className="text-xs text-(--color-text-muted) mt-0.5">{description}</p>}
          </div>
          {action}
        </div>
      )}
      {children}
    </div>
  );
}

export function Row({
  label,
  hint,
  children,
}: {
  label: ReactNode;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-4 py-3 border-b border-(--color-border) last:border-b-0">
      <div className="min-w-0">
        <p className="text-sm text-(--color-text)">{label}</p>
        {hint && <p className="text-xs text-(--color-text-dim) mt-0.5">{hint}</p>}
      </div>
      <div className="shrink-0 flex items-center gap-2">{children}</div>
    </div>
  );
}
