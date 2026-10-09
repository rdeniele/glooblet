import mascotHappy from "../assets/mascot/happy.png";
import { Button } from "./ui/Button";
import { getSettings, updateSettings } from "../lib/api";
import { useToast } from "../lib/toast";
import { SAFETY_NOTICE } from "../lib/constants";

export function FirstRunNotice({ open, onClose }: { open: boolean; onClose: () => void }) {
  const toast = useToast();
  if (!open) return null;

  const acknowledge = async () => {
    try {
      const settings = await getSettings();
      await updateSettings({ ...settings, first_run_complete: true });
    } catch (e) {
      toast.error("Could not save", String(e));
    } finally {
      onClose();
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="w-full max-w-md rounded-2xl border border-(--color-border) bg-(--color-panel) p-6 shadow-2xl shadow-black/50 text-center">
        <img src={mascotHappy} alt="" className="w-20 h-20 mx-auto mb-3" />
        <h2 className="text-lg font-semibold text-(--color-text) font-[var(--font-display)]">
          Welcome to Glooblet
        </h2>
        <p className="text-sm text-(--color-text-muted) mt-3 leading-relaxed text-left">
          {SAFETY_NOTICE}
        </p>
        <Button variant="primary" className="mt-5 w-full justify-center" onClick={acknowledge}>
          Got it, let's go
        </Button>
      </div>
    </div>
  );
}
