import { useEffect, useMemo, useState } from "react";
import { Play, Upload, Trash2, Volume2 } from "lucide-react";
import { PageHeader } from "../components/PageHeader";
import { Card, Row } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { Toggle } from "../components/ui/Toggle";
import { Slider } from "../components/ui/Slider";
import { Select } from "../components/ui/Select";
import { StatusBadge } from "../components/ui/StatusBadge";
import { Spinner } from "../components/ui/Spinner";
import { ConfirmDialog } from "../components/ui/Dialog";
import { useToast } from "../lib/toast";
import { getErrorMessage } from "../lib/errors";
import {
  listSoundPresets,
  previewSound,
  importCustomSound,
  deleteCustomSound,
  getClickSoundConfig,
  setClickSoundConfig,
  getSettings,
  updateSettings,
  setClickSoundEnabled,
  setDesktopSoundBehavior,
} from "../lib/api";
import type { SoundPreset, ClickSoundConfig, AppSettings } from "../lib/types";

const CATEGORY_ORDER = ["Soft", "Mechanical", "Retro", "Arcade", "Sci-Fi", "Minimal", "Custom"];

export function ClickStudio() {
  const toast = useToast();
  const [loading, setLoading] = useState(true);
  const [presets, setPresets] = useState<SoundPreset[]>([]);
  const [config, setConfig] = useState<ClickSoundConfig | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<SoundPreset | null>(null);
  const [importing, setImporting] = useState(false);

  const load = async () => {
    setLoading(true);
    try {
      const [presetList, clickConfig, appSettings] = await Promise.all([
        listSoundPresets(),
        getClickSoundConfig(),
        getSettings(),
      ]);
      setPresets(presetList);
      setConfig(clickConfig);
      setSettings(appSettings);
    } catch (e) {
      toast.error("Could not load Click Studio", getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const grouped = useMemo(() => {
    const groups = new Map<string, SoundPreset[]>();
    for (const p of presets) {
      const key = p.is_builtin ? p.category : "Custom";
      if (!groups.has(key)) groups.set(key, []);
      groups.get(key)!.push(p);
    }
    return CATEGORY_ORDER.map((cat) => [cat, groups.get(cat) ?? []] as const).filter(([, list]) => list.length > 0);
  }, [presets]);

  const preview = async (preset: SoundPreset) => {
    try {
      await previewSound(preset.id);
    } catch (e) {
      toast.error("Could not play preview", getErrorMessage(e));
    }
  };

  const importSound = async () => {
    setImporting(true);
    try {
      const result = await importCustomSound("Custom sound", "Custom");
      if (result) {
        setPresets((p) => [...p, result]);
        toast.success(`Imported "${result.name}"`, result.duration_ms ? `${(result.duration_ms / 1000).toFixed(2)}s` : undefined);
      }
    } catch (e) {
      toast.error("Could not import sound", getErrorMessage(e));
    } finally {
      setImporting(false);
    }
  };

  const removeSound = async () => {
    if (!confirmDelete) return;
    try {
      await deleteCustomSound(confirmDelete.id);
      setPresets((p) => p.filter((s) => s.id !== confirmDelete.id));
      toast.success("Sound removed");
    } catch (e) {
      toast.error("Could not remove sound", getErrorMessage(e));
    } finally {
      setConfirmDelete(null);
    }
  };

  const updateSlot = async (slot: keyof ClickSoundConfig, presetId: string) => {
    if (!config) return;
    const next = { ...config, [slot]: presetId || null };
    setConfig(next);
    try {
      await setClickSoundConfig(next);
    } catch (e) {
      toast.error("Could not update click sound mapping", getErrorMessage(e));
    }
  };

  const toggleEnabled = async (enabled: boolean) => {
    if (!settings) return;
    setSettings({ ...settings, click_sound_enabled: enabled });
    try {
      await setClickSoundEnabled(enabled);
      toast.success(enabled ? "Click sounds enabled" : "Click sounds disabled");
    } catch (e) {
      toast.error("Could not change click sound state", getErrorMessage(e));
      load();
    }
  };

  const changeBehavior = async (behavior: AppSettings["desktop_sound_behavior"]) => {
    if (!settings) return;
    setSettings({ ...settings, desktop_sound_behavior: behavior });
    try {
      await setDesktopSoundBehavior(behavior);
    } catch (e) {
      toast.error("Could not update behavior", getErrorMessage(e));
    }
  };

  const updateMasterVolume = async (volume: number) => {
    if (!settings) return;
    const next = { ...settings, click_sound_volume: volume };
    setSettings(next);
    try {
      await updateSettings(next);
    } catch (e) {
      toast.error("Could not update volume", getErrorMessage(e));
    }
  };

  if (loading || !config || !settings) {
    return (
      <>
        <PageHeader title="Click Studio" description="Personalize desktop click sounds." />
        <Spinner />
      </>
    );
  }

  return (
    <>
      <PageHeader
        title="Click Studio"
        description="Personalize desktop click sounds. This listens for clicks only to trigger a sound — it never modifies, delays, or blocks the click itself."
      />

      <Card title="Master control">
        <Row label="Click sounds enabled" hint="Master switch — stops playback immediately when off">
          <Toggle checked={settings.click_sound_enabled} onChange={toggleEnabled} />
        </Row>
        <Row label="Master volume">
          <div className="w-48">
            <Slider
              value={settings.click_sound_volume}
              min={0}
              max={1}
              step={0.05}
              onChange={(v) => setSettings({ ...settings, click_sound_volume: v })}
              onCommit={updateMasterVolume}
              formatValue={(v) => `${Math.round(v * 100)}%`}
            />
          </div>
        </Row>
        <Row label="When a game has focus" hint="Suppresses sounds over recognized game processes by default">
          <div className="w-56">
            <Select
              value={settings.desktop_sound_behavior}
              onChange={(v) => changeBehavior(v as AppSettings["desktop_sound_behavior"])}
              options={[
                { value: "desktop_only", label: "Mute over games (recommended)" },
                { value: "always", label: "Always play" },
                { value: "disabled", label: "Disabled entirely" },
              ]}
            />
          </div>
        </Row>
      </Card>

      <Card title="Click sound mapping" className="mt-5">
        {(["left_click_preset_id", "right_click_preset_id", "middle_click_preset_id"] as const).map((slot) => (
          <Row key={slot} label={slot === "left_click_preset_id" ? "Left click" : slot === "right_click_preset_id" ? "Right click" : "Middle click (optional)"}>
            <div className="w-56">
              <Select
                value={config[slot] ?? ""}
                onChange={(v) => updateSlot(slot, v)}
                options={[{ value: "", label: "None" }, ...presets.map((p) => ({ value: p.id, label: p.name }))]}
              />
            </div>
          </Row>
        ))}
      </Card>

      <Card
        title="Sound library"
        description="Original, royalty-free sounds generated for Glooblet — no copyrighted audio."
        action={
          <Button variant="secondary" icon={<Upload className="w-4 h-4" />} onClick={importSound} disabled={importing}>
            {importing ? "Waiting for file…" : "Import sound"}
          </Button>
        }
        className="mt-5"
      >
        <div className="space-y-5">
          {grouped.map(([category, items]) => (
            <div key={category}>
              <p className="text-xs font-medium text-(--color-text-dim) uppercase tracking-wide mb-2">{category}</p>
              <div className="grid grid-cols-2 gap-2">
                {items.map((preset) => (
                  <div
                    key={preset.id}
                    className="flex items-center gap-2 rounded-xl border border-(--color-border) bg-(--color-surface) px-3 py-2.5"
                  >
                    <button
                      onClick={() => preview(preset)}
                      className="shrink-0 w-8 h-8 rounded-full bg-(--color-accent)/15 text-(--color-accent-soft) flex items-center justify-center hover:bg-(--color-accent)/25 transition-colors"
                      aria-label={`Preview ${preset.name}`}
                    >
                      <Play className="w-3.5 h-3.5 ml-0.5" />
                    </button>
                    <div className="min-w-0 flex-1">
                      <p className="text-sm text-(--color-text) truncate">{preset.name}</p>
                      {preset.duration_ms && (
                        <p className="text-[11px] text-(--color-text-dim)">{(preset.duration_ms / 1000).toFixed(2)}s</p>
                      )}
                    </div>
                    {!preset.is_builtin && (
                      <button
                        onClick={() => setConfirmDelete(preset)}
                        className="shrink-0 text-(--color-text-dim) hover:text-(--color-error) transition-colors"
                        aria-label={`Remove ${preset.name}`}
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    )}
                  </div>
                ))}
              </div>
            </div>
          ))}
          {presets.length === 0 && (
            <p className="text-sm text-(--color-text-muted)">No sounds available.</p>
          )}
        </div>
      </Card>

      <Card title="How this works" className="mt-5">
        <p className="text-sm text-(--color-text-muted) leading-relaxed inline-flex items-start gap-1.5">
          <Volume2 className="w-4 h-4 shrink-0 mt-0.5 text-(--color-text-dim)" />
          Glooblet does not replace Windows' own click sound system and cannot override click sounds inside every
          app — there is no universal API for that. Instead it listens for a click (read-only, like a screen reader
          would) and plays your chosen sound alongside it. Nothing about the click itself is changed or delayed.
          <StatusBadge status="limited_support" />
        </p>
      </Card>

      <ConfirmDialog
        open={confirmDelete !== null}
        title={`Remove "${confirmDelete?.name}"?`}
        description="This deletes the imported sound file. Any click mapped to it will be cleared."
        confirmLabel="Remove"
        danger
        onConfirm={removeSound}
        onCancel={() => setConfirmDelete(null)}
      />
    </>
  );
}
