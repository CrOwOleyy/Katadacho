/**
 * Humeurs du fantôme, une par situation de l'application :
 * idle (repos), thinking (réflexion), alert (notification), music (média en cours),
 * focus (minuteur en concentration), sleepy (pause), happy (réussite), sad (erreur).
 */
export type GhostMood = "idle" | "thinking" | "alert" | "music" | "focus" | "sleepy" | "happy" | "sad";

export const GHOST_MOODS: GhostMood[] = ["idle", "thinking", "alert", "music", "focus", "sleepy", "happy", "sad"];
