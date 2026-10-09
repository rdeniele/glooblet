import type { ButtonHTMLAttributes, ReactNode } from "react";

type Variant = "primary" | "secondary" | "ghost" | "danger";

const VARIANTS: Record<Variant, string> = {
  primary:
    "bg-(--color-accent) text-(--color-bg) hover:bg-(--color-accent-soft) disabled:bg-(--color-surface) disabled:text-(--color-text-dim)",
  secondary:
    "bg-(--color-surface) text-(--color-text) border border-(--color-border) hover:bg-(--color-surface-hover) disabled:opacity-50",
  ghost:
    "bg-transparent text-(--color-text-muted) hover:text-(--color-text) hover:bg-(--color-surface) disabled:opacity-40",
  danger:
    "bg-(--color-error)/15 text-(--color-error) border border-(--color-error)/30 hover:bg-(--color-error)/25 disabled:opacity-50",
};

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  icon?: ReactNode;
}

export function Button({ variant = "secondary", icon, className = "", children, ...rest }: ButtonProps) {
  return (
    <button
      className={`inline-flex items-center justify-center gap-1.5 rounded-lg px-3.5 py-2 text-sm font-medium transition-colors duration-150 disabled:cursor-not-allowed ${VARIANTS[variant]} ${className}`}
      {...rest}
    >
      {icon}
      {children}
    </button>
  );
}
