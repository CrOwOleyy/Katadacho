export type TimerKind = "none" | "countdown" | "pomodoro";
export type TimerPhase = "focus" | "shortBreak" | "longBreak" | "countdown";
export type TimerStatus = "idle" | "running" | "paused" | "ready";

export interface TimerSnapshot {
  kind: TimerKind;
  phase: TimerPhase | null;
  status: TimerStatus;
  remainingSeconds: number;
  totalSeconds: number;
  completedFocusSessions: number;
  sessionsBeforeLongBreak: number;
  cycleProgress: number;
}

export interface TimerFinished {
  phase: TimerPhase;
  nextPhase: TimerPhase | null;
}
