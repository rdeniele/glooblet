import { useEffect, useState } from "react";
import { PageHeader } from "../components/PageHeader";
import { Card, Row } from "../components/ui/Card";
import { Toggle } from "../components/ui/Toggle";
import { Spinner } from "../components/ui/Spinner";
import { ConfirmDialog } from "../components/ui/Dialog";
import { useToast } from "../lib/toast";
import { getSettings, setLaunchAtStartup } from "../lib/api";
import type { AppSettings } from "../lib/types";
import { SAFETY_NOTICE } from "../lib/constants";
import mascotFocused from "../assets/mascot/focused.png";

export function Settings() {
  const toast = useToast();
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [confirmStartup, setConfirmStartup] = useState(false);

  const load = async () => {
    try {
      setSettings(await getSettings());
    } catch (e) {
      toast.error("Could not load settings", String(e));
    }
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const applyStartup = async (enabled: boolean) => {
    setConfirmStartup(false);
    if (!settings) return;
    setSettings({ ...settings, launch_at_startup: enabled });
    try {
      await setLaunchAtStartup(enabled);
      toast.success(enabled ? "Glooblet will launch at startup" : "Removed from Windows startup");
    } catch (e) {
      toast.error("Could not change startup setting", String(e));
      load();
    }
  };

  if (!settings) {
    return (
      <>
        <PageHeader title="Settings" />
        <Spinner />
      </>
    );
  }

  return (
    <>
      <PageHeader title="Settings" description="App behavior, startup, and safety information." />

      <Card title="Startup">
        <Row
          label="Launch at Windows startup"
          hint="Adds a per-user Run registry entry. No admin rights, no scheduled task."
        >
          <Toggle
            checked={settings.launch_at_startup}
            onChange={(checked) => (checked ? setConfirmStartup(true) : applyStartup(false))}
          />
        </Row>
      </Card>

      <Card title="Safety and anti-cheat" className="mt-5">
        <div className="flex gap-3">
          <img src={mascotFocused} alt="" className="w-14 h-14 shrink-0" />
          <p className="text-sm text-(--color-text-muted) leading-relaxed">{SAFETY_NOTICE}</p>
        </div>
      </Card>

      <Card title="About" className="mt-5">
        <Row label="Version">0.1.0</Row>
        <Row label="Data storage">Local only — SQLite database in your Windows user profile</Row>
        <Row label="Network access">None required for core features</Row>
      </Card>

      <ConfirmDialog
        open={confirmStartup}
        title="Launch Glooblet at startup?"
        description="Glooblet will start automatically when you sign in to Windows. Click monitoring still stays off unless you've enabled it in Click Studio."
        confirmLabel="Enable"
        onConfirm={() => applyStartup(true)}
        onCancel={() => setConfirmStartup(false)}
      />
    </>
  );
}
