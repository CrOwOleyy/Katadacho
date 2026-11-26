import type { TimerPhase } from "../../lib/types/timer";

/** `mm:ss`, ou `h:mm:ss` à partir d'une heure. */
export function formatClock(totalSeconds: number): string {
  const safe = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(safe / 3600);
  const minutes = Math.floor((safe % 3600) / 60);
  const seconds = safe % 60;
  const mm = minutes.toString().padStart(2, "0");
  const ss = seconds.toString().padStart(2, "0");
  return hours > 0 ? `${hours}:${mm}:${ss}` : `${mm}:${ss}`;
}

export const PHASE_LABEL: Record<TimerPhase, string> = {
  focus: "Focus",
  shortBreak: "Short break",
  longBreak: "Long break",
  countdown: "Timer",
};

const BREAK_COLOR = "#34d399";

/** Les phases de travail suivent la couleur d'accent, les pauses sont vertes. */
export function phaseColor(phase: TimerPhase | null): string {
  return phase === "shortBreak" || phase === "longBreak" ? BREAK_COLOR : "var(--accent-color)";
}
