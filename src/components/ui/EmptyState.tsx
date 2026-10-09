import type { ReactNode } from "react";
import mascotCurious from "../../assets/mascot/curious.png";

export function EmptyState({
  title,
  description,
  action,
  icon,
}: {
  title: string;
  description?: string;
  action?: ReactNode;
  icon?: ReactNode;
}) {
  return (
    <div className="flex flex-col items-center justify-center text-center py-12 px-6">
      {icon ?? <img src={mascotCurious} alt="" className="w-20 h-20 mb-3 opacity-90" />}
      <h3 className="text-sm font-semibold text-(--color-text)">{title}</h3>
      {description && (
        <p className="text-xs text-(--color-text-muted) mt-1.5 max-w-xs leading-relaxed">{description}</p>
      )}
      {action && <div className="mt-4">{action}</div>}
    </div>
  );
}
