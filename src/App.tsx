import { useEffect, useState } from "react";
import { Sidebar } from "./components/Sidebar";
import { Overview } from "./pages/Overview";
import { MouseSettings } from "./pages/MouseSettings";
import { GameProfiles } from "./pages/GameProfiles";
import { ClickStudio } from "./pages/ClickStudio";
import { Devices } from "./pages/Devices";
import { Settings } from "./pages/Settings";
import { FirstRunNotice } from "./components/FirstRunNotice";
import { getSettings, listProfiles } from "./lib/api";
import { useToast } from "./lib/toast";
import { getErrorMessage } from "./lib/errors";

export type Page =
  | "overview"
  | "mouse-settings"
  | "game-profiles"
  | "click-studio"
  | "devices"
  | "settings";

function App() {
  const [page, setPage] = useState<Page>("overview");
  const [activeProfileName, setActiveProfileName] = useState<string | null>(null);
  const [showFirstRun, setShowFirstRun] = useState(false);
  const [checkedFirstRun, setCheckedFirstRun] = useState(false);
  const toast = useToast();

  useEffect(() => {
    getSettings()
      .then((s) => {
        if (!s.first_run_complete) setShowFirstRun(true);
        if (s.selected_profile_id) {
          listProfiles()
            .then((profiles) => {
              const match = profiles.find((p) => p.id === s.selected_profile_id);
              setActiveProfileName(match?.name ?? null);
            })
            .catch(() => {});
        }
      })
      .catch((e) => toast.error("Could not load settings", getErrorMessage(e)))
      .finally(() => setCheckedFirstRun(true));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="flex h-screen w-screen bg-(--color-bg) text-(--color-text)">
      <Sidebar active={page} onNavigate={setPage} activeProfileName={activeProfileName} />
      <main className="flex-1 overflow-y-auto">
        <div className="max-w-5xl mx-auto px-8 py-8">
          {page === "overview" && <Overview onNavigate={setPage} />}
          {page === "mouse-settings" && <MouseSettings />}
          {page === "game-profiles" && (
            <GameProfiles onActiveProfileChange={setActiveProfileName} />
          )}
          {page === "click-studio" && <ClickStudio />}
          {page === "devices" && <Devices />}
          {page === "settings" && <Settings />}
        </div>
      </main>
      {checkedFirstRun && (
        <FirstRunNotice open={showFirstRun} onClose={() => setShowFirstRun(false)} />
      )}
    </div>
  );
}

export default App;
