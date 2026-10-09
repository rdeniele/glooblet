import { useEffect, useState } from "react";
import { RotateCcw } from "lucide-react";
import { PageHeader } from "../components/PageHeader";
import { Card, Row } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { Toggle } from "../components/ui/Toggle";
import { Slider } from "../components/ui/Slider";
import { Tooltip } from "../components/ui/Tooltip";
import { StatusBadge } from "../components/ui/StatusBadge";
import { ConfirmDialog } from "../components/ui/Dialog";
import { Spinner } from "../components/ui/Spinner";
import { useToast } from "../lib/toast";
import { getPointerConfig, applyPointerConfig, restorePointerConfig } from "../lib/api";
import type { PointerConfig } from "../lib/types";
import { POINTER_ACCELERATION_TOOLTIP, POLLING_RATE_TOOLTIP, DPI_TOOLTIP } from "../lib/constants";

const PRESETS: { name: string; description: string; speed: number; enhancePrecision: boolean }[] = [
  { name: "Precision", description: "Slow, consistent movement for lining up shots.", speed: 6, enhancePrecision: false },
  { name: "Balanced", description: "Windows' own default feel.", speed: 10, enhancePrecision: false },
  { name: "Fast", description: "Quicker cursor travel for large, multi-monitor desktops.", speed: 14, enhancePrecision: false },
  { name: "Smooth (with acceleration)", description: "Enhance pointer precision turned on.", speed: 10, enhancePrecision: true },
];

export function MouseSettings() {
  const toast = useToast();
  const [loading, setLoading] = useState(true);
  const [pointer, setPointer] = useState<PointerConfig | null>(null);
  const [draftSpeed, setDraftSpeed] = useState(10);
  const [previous, setPrevious] = useState<PointerConfig | null>(null);
  const [confirmApply, setConfirmApply] = useState<{ speed: number; enhancePrecision: boolean } | null>(null);

  const load = async () => {
    setLoading(true);
    try {
      const cfg = await getPointerConfig();
      setPointer(cfg);
      setDraftSpeed(cfg.speed);
    } catch (e) {
      toast.error("Could not read Windows pointer settings", String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const apply = async (speed: number, enhancePrecision: boolean) => {
    setConfirmApply(null);
    try {
      const result = await applyPointerConfig(speed, enhancePrecision);
      setPrevious(result.previous);
      setPointer(result.applied);
      setDraftSpeed(result.applied.speed);
      toast.success("Pointer settings applied and verified", "Windows confirmed the new values on read-back.");
    } catch (e) {
      toast.error("Could not apply pointer settings", String(e));
    }
  };

  const rollback = async () => {
    if (!previous) return;
    try {
      const restored = await restorePointerConfig(previous);
      setPointer(restored);
      setDraftSpeed(restored.speed);
      setPrevious(null);
      toast.success("Restored previous pointer settings");
    } catch (e) {
      toast.error("Could not restore previous settings", String(e));
    }
  };

  if (loading) {
    return (
      <>
        <PageHeader title="Mouse Settings" description="Configure supported mouse and Windows pointer behavior." />
        <Spinner />
      </>
    );
  }

  return (
    <>
      <PageHeader
        title="Mouse Settings"
        description="Configure supported mouse and Windows pointer behavior. Pointer changes affect Windows globally, not a single game."
      />

      <Card title="Windows pointer configuration" description="Applies system-wide, read back after every change to confirm it took effect.">
        <Row label="Pointer speed" hint="1 (slowest) – 20 (fastest)">
          <div className="w-48">
            <Slider value={draftSpeed} min={1} max={20} onChange={setDraftSpeed} />
          </div>
        </Row>
        <Row
          label={
            <span className="inline-flex items-center gap-1.5">
              Enhance pointer precision
              <Tooltip text={POINTER_ACCELERATION_TOOLTIP} />
            </span>
          }
          hint="Windows mouse acceleration"
        >
          <Toggle
            checked={pointer?.enhance_precision ?? false}
            onChange={(checked) => setConfirmApply({ speed: draftSpeed, enhancePrecision: checked })}
          />
        </Row>
        <div className="flex items-center justify-between pt-3">
          <Button
            variant="ghost"
            icon={<RotateCcw className="w-4 h-4" />}
            onClick={rollback}
            disabled={!previous}
          >
            Restore previous configuration
          </Button>
          <Button
            variant="primary"
            disabled={draftSpeed === pointer?.speed}
            onClick={() => setConfirmApply({ speed: draftSpeed, enhancePrecision: pointer?.enhance_precision ?? false })}
          >
            Apply speed change
          </Button>
        </div>
      </Card>

      <Card title="DPI" description="Hardware DPI control depends on your specific mouse." className="mt-5">
        <p className="text-sm text-(--color-text-muted) leading-relaxed inline-flex items-start gap-1.5">
          Glooblet does not change hardware DPI directly — there is no public, cross-vendor API for it. Store a
          reference DPI value per game in <strong className="text-(--color-text) mx-1">Game Profiles</strong>, matching
          whatever your mouse's own software (or onboard button) is set to.
          <Tooltip text={DPI_TOOLTIP} />
        </p>
      </Card>

      <Card title="Polling rate" className="mt-5">
        <Row label="Polling rate control">
          <StatusBadge status="unsupported" />
        </Row>
        <p className="text-sm text-(--color-text-muted) leading-relaxed mt-2 inline-flex items-start gap-1.5">
          No generic, documented Windows API exposes polling rate for arbitrary mice. If your mouse supports it,
          change it in the manufacturer's own configuration software.
          <Tooltip text={POLLING_RATE_TOOLTIP} />
        </p>
      </Card>

      <Card title="Sensitivity & mouse feel presets" description="Each preset applies a Windows pointer-speed value — confirmed before anything changes." className="mt-5">
        <div className="grid grid-cols-2 gap-3">
          {PRESETS.map((preset) => (
            <button
              key={preset.name}
              onClick={() => setConfirmApply({ speed: preset.speed, enhancePrecision: preset.enhancePrecision })}
              className="text-left rounded-xl border border-(--color-border) bg-(--color-surface) px-4 py-3 hover:bg-(--color-surface-hover) hover:border-(--color-accent)/40 transition-colors"
            >
              <p className="text-sm font-medium text-(--color-text)">{preset.name}</p>
              <p className="text-xs text-(--color-text-muted) mt-0.5">{preset.description}</p>
            </button>
          ))}
        </div>
      </Card>

      <ConfirmDialog
        open={confirmApply !== null}
        title="Apply pointer settings?"
        description={`This changes Windows pointer speed to ${confirmApply?.speed ?? ""}/20 and ${
          confirmApply?.enhancePrecision ? "enables" : "disables"
        } Enhance pointer precision, system-wide (not per-app).`}
        confirmLabel="Apply"
        onConfirm={() => confirmApply && apply(confirmApply.speed, confirmApply.enhancePrecision)}
        onCancel={() => setConfirmApply(null)}
      />
    </>
  );
}
