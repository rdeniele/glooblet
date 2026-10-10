import { useEffect, useState } from "react";
import { Mouse, Gamepad2, Music2, RotateCcw, ShieldCheck } from "lucide-react";
import { PageHeader } from "../components/PageHeader";
import { Card, Row } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { StatusBadge } from "../components/ui/StatusBadge";
import { Spinner } from "../components/ui/Spinner";
import { ConfirmDialog } from "../components/ui/Dialog";
import { useToast } from "../lib/toast";
import { getErrorMessage } from "../lib/errors";
import {
  listDevices,
  getPointerConfig,
  getSettings,
  listProfiles,
  getClickSoundConfig,
  applyPointerConfig,
  capabilityCheck,
} from "../lib/api";
import type { DeviceInfo, PointerConfig, AppSettings, MouseProfile, CapabilityReport } from "../lib/types";
import type { Page } from "../App";

export function Overview({ onNavigate }: { onNavigate: (page: Page) => void }) {
  const toast = useToast();
  const [loading, setLoading] = useState(true);
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [pointer, setPointer] = useState<PointerConfig | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [activeProfile, setActiveProfile] = useState<MouseProfile | null>(null);
  const [clickConfigured, setClickConfigured] = useState(false);
  const [confirmRestore, setConfirmRestore] = useState(false);
  const [capabilities, setCapabilities] = useState<CapabilityReport | null>(null);
  const [checkingCapabilities, setCheckingCapabilities] = useState(false);

  const load = async () => {
    setLoading(true);
    try {
      const [deviceList, pointerConfig, appSettings, profiles, clickConfig] = await Promise.all([
        listDevices(),
        getPointerConfig(),
        getSettings(),
        listProfiles(),
        getClickSoundConfig(),
      ]);
      setDevices(deviceList);
      setPointer(pointerConfig);
      setSettings(appSettings);
      setActiveProfile(profiles.find((p) => p.id === appSettings.selected_profile_id) ?? null);
      setClickConfigured(
        Boolean(
          clickConfig.left_click_preset_id ||
            clickConfig.right_click_preset_id ||
            clickConfig.middle_click_preset_id,
        ),
      );
    } catch (e) {
      toast.error("Could not load overview", getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const restoreDefaults = async () => {
    setConfirmRestore(false);
    try {
      await applyPointerConfig(10, false);
      toast.success("Restored Windows pointer defaults", "Speed set to 10, acceleration disabled.");
      load();
    } catch (e) {
      toast.error("Could not restore defaults", getErrorMessage(e));
    }
  };

  const runCapabilityCheck = async () => {
    setCheckingCapabilities(true);
    try {
      const report = await capabilityCheck();
      setCapabilities(report);
    } catch (e) {
      toast.error("Capability check failed", getErrorMessage(e));
    } finally {
      setCheckingCapabilities(false);
    }
  };

  if (loading) {
    return (
      <>
        <PageHeader title="Overview" description="Your mouse configuration at a glance." />
        <Spinner label="Reading current configuration…" />
      </>
    );
  }

  const primaryDevice = devices[0];

  return (
    <>
      <PageHeader
        title="Overview"
        description="Your mouse configuration at a glance."
        action={
          <Button variant="secondary" icon={<RotateCcw className="w-4 h-4" />} onClick={load}>
            Refresh
          </Button>
        }
      />

      <div className="grid grid-cols-2 gap-5">
        <Card title="Current device">
          {primaryDevice ? (
            <div className="space-y-0">
              <Row label="Name">{primaryDevice.name}</Row>
              <Row label="Manufacturer">{primaryDevice.manufacturer ?? "Unknown"}</Row>
              <Row label="Connection">{primaryDevice.connection_type}</Row>
              <Row label="Hardware control">
                <StatusBadge status="manual_configuration_only" />
              </Row>
            </div>
          ) : (
            <Row label="Device">
              <StatusBadge status="not_detected" />
            </Row>
          )}
        </Card>

        <Card title="Active configuration">
          <Row label="Windows pointer speed">{pointer?.speed ?? "—"} / 20</Row>
          <Row label="Enhance pointer precision">
            <StatusBadge status={pointer?.enhance_precision ? "configured" : "not_detected"} />
          </Row>
          <Row label="Active profile">{activeProfile?.name ?? "None selected"}</Row>
          <Row label="DPI (reference)">{activeProfile?.dpi ?? "—"}</Row>
          <Row label="Desktop click sounds">
            <StatusBadge
              status={settings?.click_sound_enabled && clickConfigured ? "configured" : "not_detected"}
            />
          </Row>
        </Card>
      </div>

      <Card title="Quick actions" className="mt-5">
        <div className="grid grid-cols-2 gap-3">
          <Button variant="secondary" icon={<Mouse className="w-4 h-4" />} onClick={() => onNavigate("mouse-settings")}>
            Open Mouse Settings
          </Button>
          <Button variant="secondary" icon={<Gamepad2 className="w-4 h-4" />} onClick={() => onNavigate("game-profiles")}>
            Open Game Profiles
          </Button>
          <Button variant="secondary" icon={<Music2 className="w-4 h-4" />} onClick={() => onNavigate("click-studio")}>
            Open Click Studio
          </Button>
          <Button variant="secondary" icon={<RotateCcw className="w-4 h-4" />} onClick={() => setConfirmRestore(true)}>
            Restore Glooblet defaults
          </Button>
        </div>
        <div className="mt-3 pt-3 border-t border-(--color-border)">
          <Button
            variant="ghost"
            icon={<ShieldCheck className="w-4 h-4" />}
            onClick={runCapabilityCheck}
            disabled={checkingCapabilities}
          >
            {checkingCapabilities ? "Checking…" : "Run configuration capability check"}
          </Button>
          {capabilities && (
            <div className="mt-3 grid grid-cols-2 gap-x-6 gap-y-1 text-xs text-(--color-text-muted)">
              <span>Pointer settings readable: <StatusBadge status={capabilities.pointer_settings_readable ? "supported" : "unsupported"} /></span>
              <span>Pointer settings writable: <StatusBadge status={capabilities.pointer_settings_writable ? "supported" : "unsupported"} /></span>
              <span>Device list available: <StatusBadge status={capabilities.device_enumeration_available ? "supported" : "not_detected"} /></span>
              <span>Audio output available: <StatusBadge status={capabilities.audio_output_available ? "supported" : "unsupported"} /></span>
              <span>Startup registry writable: <StatusBadge status={capabilities.startup_registry_writable ? "supported" : "unsupported"} /></span>
            </div>
          )}
        </div>
      </Card>

      <ConfirmDialog
        open={confirmRestore}
        title="Restore Glooblet-managed defaults?"
        description="This sets Windows pointer speed to 10/20 and turns off Enhance pointer precision. This is a global Windows setting change, not limited to one app."
        confirmLabel="Restore defaults"
        onConfirm={restoreDefaults}
        onCancel={() => setConfirmRestore(false)}
      />
    </>
  );
}
