import type { ReactNode } from "react";

export function PageHeader({
  title,
  description,
  action,
}: {
  title: string;
  description?: string;
  action?: ReactNode;
}) {
  return (
    <div className="flex items-start justify-between gap-4 mb-6">
      <div>
        <h1 className="text-xl font-semibold text-(--color-text) font-[var(--font-display)]">{title}</h1>
        {description && <p className="text-sm text-(--color-text-muted) mt-1 max-w-xl">{description}</p>}
      </div>
      {action}
    </div>
  );
}
