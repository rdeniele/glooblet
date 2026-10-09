import { createContext, useCallback, useContext, useRef, useState, type ReactNode } from "react";
import { CheckCircle2, XCircle, Info, X } from "lucide-react";

type ToastKind = "success" | "error" | "info";

interface Toast {
  id: number;
  kind: ToastKind;
  title: string;
  description?: string;
}

interface ToastContextValue {
  push: (kind: ToastKind, title: string, description?: string) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

export function useToast() {
  const ctx = useContext(ToastContext);
  if (!ctx) throw new Error("useToast must be used within ToastProvider");
  return {
    success: (title: string, description?: string) => ctx.push("success", title, description),
    error: (title: string, description?: string) => ctx.push("error", title, description),
    info: (title: string, description?: string) => ctx.push("info", title, description),
  };
}

const ICONS: Record<ToastKind, ReactNode> = {
  success: <CheckCircle2 className="w-5 h-5 text-(--color-success)" />,
  error: <XCircle className="w-5 h-5 text-(--color-error)" />,
  info: <Info className="w-5 h-5 text-(--color-cyan)" />,
};

const BORDER: Record<ToastKind, string> = {
  success: "border-(--color-success)/40",
  error: "border-(--color-error)/40",
  info: "border-(--color-cyan)/40",
};

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const nextId = useRef(0);

  const push = useCallback((kind: ToastKind, title: string, description?: string) => {
    const id = nextId.current++;
    setToasts((t) => [...t, { id, kind, title, description }]);
    window.setTimeout(() => {
      setToasts((t) => t.filter((toast) => toast.id !== id));
    }, 5000);
  }, []);

  const dismiss = (id: number) => setToasts((t) => t.filter((toast) => toast.id !== id));

  return (
    <ToastContext.Provider value={{ push }}>
      {children}
      <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 w-80 pointer-events-none">
        {toasts.map((t) => (
          <div
            key={t.id}
            role="status"
            className={`pointer-events-auto flex items-start gap-2.5 rounded-xl border ${BORDER[t.kind]} bg-(--color-panel) px-3.5 py-3 shadow-lg shadow-black/40 animate-[toast-in_180ms_ease-out]`}
          >
            <div className="shrink-0 mt-0.5">{ICONS[t.kind]}</div>
            <div className="min-w-0 flex-1">
              <p className="text-sm font-medium text-(--color-text) leading-snug">{t.title}</p>
              {t.description && (
                <p className="text-xs text-(--color-text-muted) mt-0.5 leading-snug">{t.description}</p>
              )}
            </div>
            <button
              onClick={() => dismiss(t.id)}
              className="shrink-0 text-(--color-text-dim) hover:text-(--color-text) transition-colors"
              aria-label="Dismiss notification"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        ))}
      </div>
      <style>{`
        @keyframes toast-in {
          from { opacity: 0; transform: translateY(8px); }
          to { opacity: 1; transform: translateY(0); }
        }
      `}</style>
    </ToastContext.Provider>
  );
}
