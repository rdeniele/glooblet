type Status =
  | "supported"
  | "configured"
  | "read_only"
  | "not_detected"
  | "unsupported"
  | "requires_confirmation"
  | "manual_configuration_only"
  | "reviewed"
  | "limited_support"
  | "under_review"
  | "disabled";

const LABELS: Record<Status, string> = {
  supported: "Supported",
  configured: "Configured",
  read_only: "Read-only",
  not_detected: "Not detected",
  unsupported: "Unsupported",
  requires_confirmation: "Requires confirmation",
  manual_configuration_only: "Manual configuration only",
  reviewed: "Reviewed",
  limited_support: "Limited support",
  under_review: "Under review",
  disabled: "Disabled",
};

const STYLES: Record<Status, string> = {
  supported: "bg-(--color-success)/15 text-(--color-success) border-(--color-success)/30",
  configured: "bg-(--color-accent)/15 text-(--color-accent-soft) border-(--color-accent)/30",
  read_only: "bg-(--color-cyan)/15 text-(--color-cyan) border-(--color-cyan)/30",
  not_detected: "bg-(--color-surface) text-(--color-text-muted) border-(--color-border)",
  unsupported: "bg-(--color-error)/10 text-(--color-error) border-(--color-error)/25",
  requires_confirmation: "bg-(--color-warning)/15 text-(--color-warning) border-(--color-warning)/30",
  manual_configuration_only: "bg-(--color-surface) text-(--color-text-muted) border-(--color-border)",
  reviewed: "bg-(--color-success)/15 text-(--color-success) border-(--color-success)/30",
  limited_support: "bg-(--color-warning)/15 text-(--color-warning) border-(--color-warning)/30",
  under_review: "bg-(--color-cyan)/15 text-(--color-cyan) border-(--color-cyan)/30",
  disabled: "bg-(--color-surface) text-(--color-text-dim) border-(--color-border)",
};

export function StatusBadge({ status }: { status: Status }) {
  return (
    <span
      className={`inline-flex items-center rounded-full border px-2.5 py-0.5 text-[11px] font-medium whitespace-nowrap ${STYLES[status]}`}
    >
      {LABELS[status]}
    </span>
  );
}
