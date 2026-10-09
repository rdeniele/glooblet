import { LayoutDashboard, Mouse, Gamepad2, Music2, Usb, Settings as SettingsIcon } from "lucide-react";
import mascotFront from "../assets/mascot/front.png";
import type { Page } from "../App";

const NAV: { id: Page; label: string; icon: typeof LayoutDashboard }[] = [
  { id: "overview", label: "Overview", icon: LayoutDashboard },
  { id: "mouse-settings", label: "Mouse Settings", icon: Mouse },
  { id: "game-profiles", label: "Game Profiles", icon: Gamepad2 },
  { id: "click-studio", label: "Click Studio", icon: Music2 },
  { id: "devices", label: "Devices", icon: Usb },
  { id: "settings", label: "Settings", icon: SettingsIcon },
];

export function Sidebar({
  active,
  onNavigate,
  activeProfileName,
}: {
  active: Page;
  onNavigate: (page: Page) => void;
  activeProfileName: string | null;
}) {
  return (
    <nav className="w-56 shrink-0 bg-(--color-sidebar) border-r border-(--color-border) flex flex-col">
      <div className="flex items-center gap-2.5 px-4 py-5">
        <img src={mascotFront} alt="" className="w-9 h-9 object-contain" />
        <span className="font-[var(--font-display)] font-bold text-lg tracking-wide text-(--color-text)">
          Glooblet
        </span>
      </div>

      <div className="flex-1 px-2.5 flex flex-col gap-0.5">
        {NAV.map(({ id, label, icon: Icon }) => {
          const isActive = active === id;
          return (
            <button
              key={id}
              onClick={() => onNavigate(id)}
              aria-current={isActive ? "page" : undefined}
              className={`flex items-center gap-2.5 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors duration-150 ${
                isActive
                  ? "bg-(--color-accent)/15 text-(--color-accent-soft)"
                  : "text-(--color-text-muted) hover:bg-(--color-surface) hover:text-(--color-text)"
              }`}
            >
              <Icon className="w-4 h-4 shrink-0" />
              {label}
            </button>
          );
        })}
      </div>

      <div className="px-4 py-4 border-t border-(--color-border)">
        <p className="text-[11px] uppercase tracking-wide text-(--color-text-dim) mb-1">Active profile</p>
        <p className="text-sm text-(--color-text) truncate">{activeProfileName ?? "None selected"}</p>
      </div>
    </nav>
  );
}
