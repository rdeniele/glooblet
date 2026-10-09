import { Loader2 } from "lucide-react";

export function Spinner({ label }: { label?: string }) {
  return (
    <div className="flex items-center gap-2 text-(--color-text-muted) text-sm py-8 justify-center">
      <Loader2 className="w-4 h-4 animate-spin" />
      {label ?? "Loading…"}
    </div>
  );
}
