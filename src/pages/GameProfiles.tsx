import { useEffect, useMemo, useState } from "react";
import { Plus, Upload, Download, Copy, Trash2, Pencil, ArrowLeft, CheckCircle2, ShieldQuestion } from "lucide-react";
import { PageHeader } from "../components/PageHeader";
import { Card, Row } from "../components/ui/Card";
import { Button } from "../components/ui/Button";
import { NumberInput } from "../components/ui/NumberInput";
import { Select } from "../components/ui/Select";
import { StatusBadge } from "../components/ui/StatusBadge";
import { ConfirmDialog } from "../components/ui/Dialog";
import { EmptyState } from "../components/ui/EmptyState";
import { Spinner } from "../components/ui/Spinner";
import { Tooltip } from "../components/ui/Tooltip";
import { useToast } from "../lib/toast";
import {
  listProfiles,
  createProfile,
  updateProfile,
  deleteProfile,
  duplicateProfile,
  exportProfileToFile,
  importProfileFromFile,
  listKnownGames,
  gameFormula,
  calculateCmPer360,
  compatibilityRegistry,
  getSettings,
  updateSettings,
} from "../lib/api";
import type { MouseProfile, GameFormula, Cm360Result, IntegrationStatus } from "../lib/types";
import { CM_360_TOOLTIP } from "../lib/constants";

type DraftProfile = Omit<MouseProfile, "id" | "created_at" | "updated_at" | "schema_version"> & {
  id?: string;
};

const EMPTY_DRAFT: DraftProfile = {
  name: "",
  game: null,
  dpi: 800,
  sensitivity: 1,
  yaw_constant: null,
  pointer_speed: null,
  enhance_precision: null,
  notes: null,
};

export function GameProfiles({
  onActiveProfileChange,
}: {
  onActiveProfileChange: (name: string | null) => void;
}) {
  const toast = useToast();
  const [loading, setLoading] = useState(true);
  const [profiles, setProfiles] = useState<MouseProfile[]>([]);
  const [activeProfileId, setActiveProfileId] = useState<string | null>(null);
  const [games, setGames] = useState<string[]>([]);
  const [editing, setEditing] = useState<DraftProfile | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<MouseProfile | null>(null);
  const [showRegistry, setShowRegistry] = useState(false);
  const [registry, setRegistry] = useState<IntegrationStatus[]>([]);

  const load = async () => {
    setLoading(true);
    try {
      const [profileList, gameList, settings] = await Promise.all([
        listProfiles(),
        listKnownGames(),
        getSettings(),
      ]);
      setProfiles(profileList);
      setGames(gameList);
      setActiveProfileId(settings.selected_profile_id);
    } catch (e) {
      toast.error("Could not load game profiles", String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const setActive = async (profile: MouseProfile) => {
    try {
      const settings = await getSettings();
      await updateSettings({ ...settings, selected_profile_id: profile.id });
      setActiveProfileId(profile.id);
      onActiveProfileChange(profile.name);
      toast.success(`"${profile.name}" is now the active profile`);
    } catch (e) {
      toast.error("Could not set active profile", String(e));
    }
  };

  const save = async (draft: DraftProfile) => {
    try {
      if (draft.id) {
        const full = profiles.find((p) => p.id === draft.id)!;
        const updated = await updateProfile({ ...full, ...draft, id: draft.id });
        setProfiles((ps) => ps.map((p) => (p.id === updated.id ? updated : p)));
        if (updated.id === activeProfileId) onActiveProfileChange(updated.name);
        toast.success("Profile saved");
      } else {
        const created = await createProfile(draft);
        setProfiles((ps) => [created, ...ps]);
        toast.success("Profile created");
      }
      setEditing(null);
    } catch (e) {
      toast.error("Could not save profile", String(e));
    }
  };

  const remove = async () => {
    if (!confirmDelete) return;
    try {
      await deleteProfile(confirmDelete.id);
      setProfiles((ps) => ps.filter((p) => p.id !== confirmDelete.id));
      if (confirmDelete.id === activeProfileId) {
        setActiveProfileId(null);
        onActiveProfileChange(null);
      }
      toast.success("Profile deleted");
    } catch (e) {
      toast.error("Could not delete profile", String(e));
    } finally {
      setConfirmDelete(null);
    }
  };

  const duplicate = async (profile: MouseProfile) => {
    try {
      const dup = await duplicateProfile(profile.id, `${profile.name} copy`);
      setProfiles((ps) => [dup, ...ps]);
      toast.success("Profile duplicated");
    } catch (e) {
      toast.error("Could not duplicate profile", String(e));
    }
  };

  const exportOne = async (profile: MouseProfile) => {
    try {
      const path = await exportProfileToFile(profile.id, profile.name.replace(/[^a-z0-9-_]+/gi, "_"));
      if (path) toast.success("Profile exported", path);
    } catch (e) {
      toast.error("Could not export profile", String(e));
    }
  };

  const importOne = async () => {
    try {
      const imported = await importProfileFromFile();
      if (imported) {
        setProfiles((ps) => [imported, ...ps]);
        toast.success(`Imported "${imported.name}"`);
      }
    } catch (e) {
      toast.error("Could not import profile", String(e));
    }
  };

  const openRegistry = async () => {
    setShowRegistry(true);
    if (registry.length === 0) {
      try {
        setRegistry(await compatibilityRegistry());
      } catch (e) {
        toast.error("Could not load compatibility registry", String(e));
      }
    }
  };

  if (editing) {
    return (
      <ProfileEditor
        draft={editing}
        games={games}
        onCancel={() => setEditing(null)}
        onSave={save}
      />
    );
  }

  return (
    <>
      <PageHeader
        title="Game Profiles"
        description="Store DPI, sensitivity, and notes per game. Glooblet never applies these inside a game automatically."
        action={
          <div className="flex gap-2">
            <Button variant="ghost" icon={<ShieldQuestion className="w-4 h-4" />} onClick={openRegistry}>
              Compatibility
            </Button>
            <Button variant="secondary" icon={<Upload className="w-4 h-4" />} onClick={importOne}>
              Import
            </Button>
            <Button
              variant="primary"
              icon={<Plus className="w-4 h-4" />}
              onClick={() => setEditing({ ...EMPTY_DRAFT })}
            >
              New profile
            </Button>
          </div>
        }
      />

      {loading ? (
        <Spinner />
      ) : profiles.length === 0 ? (
        <EmptyState
          title="No profiles yet"
          description="Create a profile for each game you tune sensitivity for — or import one you exported earlier."
          action={
            <Button variant="primary" icon={<Plus className="w-4 h-4" />} onClick={() => setEditing({ ...EMPTY_DRAFT })}>
              Create your first profile
            </Button>
          }
        />
      ) : (
        <div className="grid grid-cols-2 gap-4">
          {profiles.map((profile) => (
            <Card key={profile.id} className={profile.id === activeProfileId ? "border-(--color-accent)/50" : ""}>
              <div className="flex items-start justify-between gap-2">
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <h3 className="text-sm font-semibold text-(--color-text) truncate">{profile.name}</h3>
                    {profile.id === activeProfileId && (
                      <span title="Active profile">
                        <CheckCircle2 className="w-4 h-4 text-(--color-accent) shrink-0" />
                      </span>
                    )}
                  </div>
                  <p className="text-xs text-(--color-text-muted) mt-0.5">{profile.game ?? "No game set"}</p>
                </div>
              </div>

              <div className="mt-3 grid grid-cols-2 gap-x-3 gap-y-1 text-xs text-(--color-text-muted)">
                <span>DPI: <span className="text-(--color-text)">{profile.dpi ?? "—"}</span></span>
                <span>Sensitivity: <span className="text-(--color-text)">{profile.sensitivity ?? "—"}</span></span>
              </div>

              <div className="flex items-center gap-1.5 mt-4 pt-3 border-t border-(--color-border)">
                {profile.id !== activeProfileId && (
                  <Button variant="secondary" className="flex-1" onClick={() => setActive(profile)}>
                    Set active
                  </Button>
                )}
                <Button variant="ghost" onClick={() => setEditing(profile)} aria-label="Edit profile">
                  <Pencil className="w-4 h-4" />
                </Button>
                <Button variant="ghost" onClick={() => duplicate(profile)} aria-label="Duplicate profile">
                  <Copy className="w-4 h-4" />
                </Button>
                <Button variant="ghost" onClick={() => exportOne(profile)} aria-label="Export profile">
                  <Download className="w-4 h-4" />
                </Button>
                <Button variant="ghost" onClick={() => setConfirmDelete(profile)} aria-label="Delete profile">
                  <Trash2 className="w-4 h-4 text-(--color-error)" />
                </Button>
              </div>
            </Card>
          ))}
        </div>
      )}

      <ConfirmDialog
        open={confirmDelete !== null}
        title={`Delete "${confirmDelete?.name}"?`}
        description="This permanently removes the profile. This can't be undone."
        confirmLabel="Delete"
        danger
        onConfirm={remove}
        onCancel={() => setConfirmDelete(null)}
      />

      {showRegistry && (
        <CompatibilityRegistryPanel registry={registry} onClose={() => setShowRegistry(false)} />
      )}
    </>
  );
}

function ProfileEditor({
  draft: initialDraft,
  games,
  onCancel,
  onSave,
}: {
  draft: DraftProfile;
  games: string[];
  onCancel: () => void;
  onSave: (draft: DraftProfile) => void;
}) {
  const toast = useToast();
  const [draft, setDraft] = useState(initialDraft);
  const [formula, setFormula] = useState<GameFormula | null>(null);
  const [cm360, setCm360] = useState<Cm360Result | null>(null);
  const [cm360Error, setCm360Error] = useState<string | null>(null);

  useEffect(() => {
    if (!draft.game) {
      setFormula(null);
      return;
    }
    gameFormula(draft.game).then(setFormula).catch(() => setFormula(null));
  }, [draft.game]);

  useEffect(() => {
    if (!draft.game || !draft.dpi || !draft.sensitivity) {
      setCm360(null);
      setCm360Error(null);
      return;
    }
    const customYaw = draft.yaw_constant ?? undefined;
    calculateCmPer360(draft.dpi, draft.sensitivity, draft.game, customYaw)
      .then((r) => {
        setCm360(r);
        setCm360Error(null);
      })
      .catch((e) => {
        setCm360(null);
        setCm360Error(typeof e === "string" ? e : String(e));
      });
  }, [draft.game, draft.dpi, draft.sensitivity, draft.yaw_constant]);

  const gameOptions = useMemo(
    () => games.map((g) => ({ value: g, label: g })),
    [games],
  );

  const needsCustomYaw = formula?.confidence === "not_applicable" && !draft.yaw_constant;

  const handleSave = () => {
    if (!draft.name.trim()) {
      toast.error("Name required", "Give the profile a short name before saving.");
      return;
    }
    onSave(draft);
  };

  return (
    <>
      <PageHeader
        title={draft.id ? "Edit profile" : "New profile"}
        action={
          <Button variant="ghost" icon={<ArrowLeft className="w-4 h-4" />} onClick={onCancel}>
            Back to profiles
          </Button>
        }
      />

      <Card title="Profile details">
        <Row label="Name">
          <input
            autoFocus
            value={draft.name}
            onChange={(e) => setDraft({ ...draft, name: e.target.value })}
            placeholder="e.g. CS2 — Competitive"
            className="w-64 rounded-lg bg-(--color-surface) border border-(--color-border) px-3 py-2 text-sm text-(--color-text) focus:border-(--color-accent) outline-none"
          />
        </Row>
        <Row label="Game">
          <div className="w-64">
            <Select
              value={draft.game ?? ""}
              onChange={(v) => setDraft({ ...draft, game: v || null })}
              options={gameOptions}
              placeholder="Select a game"
            />
          </div>
        </Row>
        <Row label="DPI" hint="Reference value — set this on your mouse itself">
          <div className="w-32">
            <NumberInput value={draft.dpi} onChange={(v) => setDraft({ ...draft, dpi: v })} min={1} max={32000} />
          </div>
        </Row>
        <Row label="In-game sensitivity">
          <div className="w-32">
            <NumberInput value={draft.sensitivity} onChange={(v) => setDraft({ ...draft, sensitivity: v })} min={0} />
          </div>
        </Row>
        <Row label="Notes">
          <textarea
            value={draft.notes ?? ""}
            onChange={(e) => setDraft({ ...draft, notes: e.target.value || null })}
            rows={2}
            className="w-64 rounded-lg bg-(--color-surface) border border-(--color-border) px-3 py-2 text-sm text-(--color-text) focus:border-(--color-accent) outline-none resize-none"
          />
        </Row>
      </Card>

      <Card
        title="cm / 360° calculator"
        description="Physical mousepad distance for a full turn, given DPI and sensitivity."
        className="mt-5"
      >
        {formula && (
          <div className="mb-3 flex items-center gap-2 text-xs">
            <StatusBadge
              status={
                formula.confidence === "high"
                  ? "reviewed"
                  : formula.confidence === "community"
                    ? "limited_support"
                    : "manual_configuration_only"
              }
            />
            <span className="text-(--color-text-muted)">{formula.note}</span>
          </div>
        )}

        <Row
          label={
            <span className="inline-flex items-center gap-1.5">
              Custom yaw constant (advanced)
              <Tooltip text="Degrees turned per raw mouse count at sensitivity 1. Leave blank to use the game's known constant, if any." />
            </span>
          }
        >
          <div className="w-32">
            <NumberInput
              value={draft.yaw_constant}
              onChange={(v) => setDraft({ ...draft, yaw_constant: v })}
              placeholder={formula?.yaw_constant ? String(formula.yaw_constant) : "required"}
            />
          </div>
        </Row>

        <div className="mt-3 rounded-xl bg-(--color-surface) px-4 py-3">
          {cm360 ? (
            <p className="text-sm text-(--color-text) inline-flex items-center gap-1.5">
              <strong className="text-lg">{cm360.cm_per_360.toFixed(1)} cm</strong> per 360° turn
              <Tooltip text={CM_360_TOOLTIP} />
            </p>
          ) : cm360Error ? (
            <p className="text-xs text-(--color-warning)">{cm360Error}</p>
          ) : (
            <p className="text-xs text-(--color-text-dim)">Set DPI, sensitivity, and a game to calculate.</p>
          )}
          {needsCustomYaw && (
            <p className="text-xs text-(--color-text-dim) mt-1">
              This game has no documented conversion constant — enter one above if you've measured your own.
            </p>
          )}
        </div>
      </Card>

      <div className="flex justify-end gap-2 mt-5">
        <Button variant="ghost" onClick={onCancel}>
          Cancel
        </Button>
        <Button variant="primary" onClick={handleSave}>
          Save profile
        </Button>
      </div>
    </>
  );
}

function CompatibilityRegistryPanel({
  registry,
  onClose,
}: {
  registry: IntegrationStatus[];
  onClose: () => void;
}) {
  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/50 backdrop-blur-sm" onClick={onClose}>
      <div
        className="w-full max-w-2xl max-h-[80vh] overflow-y-auto rounded-2xl border border-(--color-border) bg-(--color-panel) p-6 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-start justify-between mb-4">
          <div>
            <h2 className="text-base font-semibold text-(--color-text)">Game compatibility registry</h2>
            <p className="text-xs text-(--color-text-muted) mt-1">
              Reviewed state per game. Every entry is manual-apply — Glooblet never reads game memory or injects code.
            </p>
          </div>
          <Button variant="ghost" onClick={onClose}>
            Close
          </Button>
        </div>
        <div className="space-y-3">
          {registry.length === 0 && <Spinner />}
          {registry.map((entry) => (
            <div key={entry.subject} className="rounded-xl border border-(--color-border) bg-(--color-surface) p-4">
              <div className="flex items-center justify-between gap-2">
                <h3 className="text-sm font-medium text-(--color-text)">{entry.subject}</h3>
                <StatusBadge status={entry.status as "manual_configuration_only"} />
              </div>
              <p className="text-xs text-(--color-text-muted) mt-1.5">{entry.method}</p>
              <p className="text-[11px] text-(--color-text-dim) mt-1">
                Source: {entry.source} · Last reviewed {entry.last_reviewed}
              </p>
              {entry.limitations.length > 0 && (
                <ul className="mt-2 list-disc list-inside text-[11px] text-(--color-text-dim) space-y-0.5">
                  {entry.limitations.map((l) => (
                    <li key={l}>{l}</li>
                  ))}
                </ul>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
