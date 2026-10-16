import { Ghost } from "../mascot/Ghost";
import type { GhostMood } from "../mascot/mood";
import { useSettings } from "../../store/SettingsContext";

interface SoulProps {
  /** Hauteur du fantôme en pixels. */
  size?: number;
  mood?: GhostMood;
}

/** La « Soul » de l'Island : la mascotte, animée selon les réglages d'animation. */
export function Soul({ size = 30, mood = "idle" }: SoulProps) {
  const { settings } = useSettings();
  const { enabled, speed } = settings.animations;

  return <Ghost size={size} mood={mood} animated={enabled} speed={speed} />;
}
