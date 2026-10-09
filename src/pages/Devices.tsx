import { useEffect, useState } from "react";
import { RotateCcw, Mouse } from "lucide-react";
import { PageHeader } from "../components/PageHeader";
import { Card } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { StatusBadge } from "../components/ui/StatusBadge";
import { EmptyState } from "../components/ui/EmptyState";
import { Spinner } from "../components/ui/Spinner";
import { useToast } from "../lib/toast";
import { listDevices } from "../lib/api";
import type { DeviceInfo } from "../lib/types";

const CAPABILITY_LABELS: { key: keyof DeviceInfo["capabilities"]; label: string }[] = [
  { key: "read_dpi", label: "Read DPI" },
  { key: "set_dpi", label: "Set DPI" },
  { key: "read_polling_rate", label: "Read polling rate" },
  { key: "set_polling_rate", label: "Set polling rate" },
  { key: "read_onboard_profile", label: "Read onboard profile" },
  { key: "set_onboard_profile", label: "Set onboard profile" },
  { key: "read_button_config", label: "Read button config" },
  { key: "set_button_config", label: "Set button config" },
];

export function Devices() {
  const toast = useToast();
  const [loading, setLoading] = useState(true);
  const [devices, setDevices] = useState<DeviceInfo[]>([]);

  const load = async () => {
    setLoading(true);
    try {
      setDevices(await listDevices());
    } catch (e) {
      toast.error("Could not enumerate devices", String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <>
      <PageHeader
        title="Devices"
        description="Read-only identification via Windows' own device APIs — no admin rights required."
        action={
          <Button variant="secondary" icon={<RotateCcw className="w-4 h-4" />} onClick={load}>
            Rescan
          </Button>
        }
      />

      {loading ? (
        <Spinner label="Enumerating mouse devices…" />
      ) : devices.length === 0 ? (
        <EmptyState
          title="No mice detected"
          description="Glooblet couldn't enumerate any pointing devices through Windows' SetupAPI. Your mouse still works — this only affects device identification in this app."
        />
      ) : (
        <div className="space-y-4">
          {devices.map((device) => (
            <Card key={device.id}>
              <div className="flex items-start gap-3">
                <div className="w-10 h-10 rounded-xl bg-(--color-surface) flex items-center justify-center shrink-0">
                  <Mouse className="w-5 h-5 text-(--color-accent-soft)" />
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center justify-between gap-2">
                    <h3 className="text-sm font-semibold text-(--color-text) truncate">{device.name}</h3>
                    <StatusBadge status="read_only" />
                  </div>
                  <p className="text-xs text-(--color-text-muted) mt-0.5">
                    {device.manufacturer ?? "Unknown manufacturer"} · {device.connection_type}
                  </p>
                </div>
              </div>

              <div className="mt-4 pt-3 border-t border-(--color-border) grid grid-cols-2 gap-x-6 gap-y-2">
                {CAPABILITY_LABELS.map(({ key, label }) => (
                  <div key={key} className="flex items-center justify-between text-xs">
                    <span className="text-(--color-text-muted)">{label}</span>
                    <StatusBadge status={device.capabilities[key] ? "supported" : "unsupported"} />
                  </div>
                ))}
              </div>
              <p className="text-xs text-(--color-text-dim) mt-3">
                Per-vendor DPI, polling rate, and onboard profile control have no public, cross-vendor API — use
                your mouse manufacturer's own software for those, and store the values you land on in a Game
                Profile for reference.
              </p>
            </Card>
          ))}
        </div>
      )}
    </>
  );
}
