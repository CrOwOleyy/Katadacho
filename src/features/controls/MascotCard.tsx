import { Ghost } from "../../components/mascot/Ghost";
import type { GhostMood } from "../../components/mascot/mood";
import { useSettings } from "../../store/SettingsContext";

/** Ce que la mascotte est en train de faire, en quelques mots. */
const CAPTION: Record<GhostMood, string> = {
  idle: "Resting",
  thinking: "Thinking",
  alert: "New notification",
  music: "Listening",
  focus: "Focused",
  sleepy: "Taking a break",
  happy: "Well done",
  sad: "Oops",
  watching: "Watching",
};

const GHOST_SIZE = 118;

export function MascotCard({ mood }: { mood: GhostMood }) {
  const { settings } = useSettings();
  const { enabled, speed } = settings.animations;

  return (
    <section className="col-span-2 flex flex-col items-center justify-center gap-2 rounded-2xl border border-white/10 bg-white/[0.04] p-4 backdrop-blur-2xl">
      <Ghost size={GHOST_SIZE} mood={mood} animated={enabled} speed={speed} />
      <p className="text-xs font-medium text-white/60">{CAPTION[mood]}</p>
    </section>
  );
}
