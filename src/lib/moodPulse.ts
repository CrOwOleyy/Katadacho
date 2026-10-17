import type { GhostMood } from "../components/mascot/mood";

const EVENT = "katadacho-mood-pulse";
export const DEFAULT_PULSE_MS = 4000;

interface PulseDetail {
  mood: GhostMood;
  durationMs: number;
}

/**
 * Fait réagir la mascotte quelques secondes (content après un succès, triste après une erreur).
 * N'importe quelle partie de l'interface peut l'appeler, sans connaître l'Island.
 */
export function pulseMood(mood: GhostMood, durationMs: number = DEFAULT_PULSE_MS) {
  window.dispatchEvent(new CustomEvent<PulseDetail>(EVENT, { detail: { mood, durationMs } }));
}

export function onMoodPulse(handler: (detail: PulseDetail) => void): () => void {
  const listener = (event: Event) => handler((event as CustomEvent<PulseDetail>).detail);
  window.addEventListener(EVENT, listener);
  return () => window.removeEventListener(EVENT, listener);
}
