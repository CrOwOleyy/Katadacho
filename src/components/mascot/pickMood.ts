import type { GhostMood } from "./mood";

export interface MoodInputs {
  /** Une notification est affichée. */
  notification: boolean;
  /** Humeur ponctuelle demandée par l'interface (succès, erreur). */
  pulse: GhostMood | null;
  /** Une phase du minuteur vient de se terminer. */
  timerFinished: boolean;
  timerRunning: boolean;
  timerPhase: "focus" | "shortBreak" | "longBreak" | "countdown" | null;
  /** Une application occupe tout l'écran. */
  fullscreenApp: boolean;
  mediaPlaying: boolean;
  /** Personne n'utilise le clavier ni la souris. */
  userIdle: boolean;
  /** On est dans les heures de nuit réglées. */
  night: boolean;
}

/**
 * Choisit l'humeur de la mascotte. L'ordre est celui de l'importance : ce qui vient d'arriver passe
 * avant l'état de fond (concentration, sommeil, musique).
 */
export function pickMood(input: MoodInputs): GhostMood {
  if (input.notification) return "alert";
  if (input.pulse) return input.pulse;
  if (input.timerFinished) return "happy";

  const onBreak = input.timerPhase === "shortBreak" || input.timerPhase === "longBreak";
  if (input.timerRunning && input.timerPhase === "focus") return "focus";
  if (input.fullscreenApp) return "focus";
  if (input.timerRunning && onBreak) return "sleepy";
  if (input.mediaPlaying) return "music";
  if (input.userIdle || input.night) return "sleepy";
  return "idle";
}
