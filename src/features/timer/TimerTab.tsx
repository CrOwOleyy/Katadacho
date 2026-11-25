import { Pause, Play, RotateCcw, SkipForward, Timer } from "lucide-react";
import { useState } from "react";
import { BentoCard } from "../../components/ui/BentoCard";
import { ProgressRing } from "../../components/ui/ProgressRing";
import { ipc } from "../../lib/ipc";
import type { TimerFinished, TimerSnapshot } from "../../lib/types/timer";
import { useSettings } from "../../store/SettingsContext";
import { formatClock, PHASE_LABEL, phaseColor } from "./format";

type Mode = "pomodoro" | "countdown";

const PRESET_MINUTES = [1, 5, 10, 15, 30, 60];
const MAX_MINUTES = 600;

interface TimerTabProps {
  snapshot: TimerSnapshot;
  finished: TimerFinished | null;
}

export function TimerTab({ snapshot, finished }: TimerTabProps) {
  const { settings } = useSettings();
  const [chosenMode, setChosenMode] = useState<Mode>("pomodoro");
  const [minutes, setMinutes] = useState(10);
  const [error, setError] = useState<string | null>(null);

  const active = snapshot.kind !== "none";
  // Pendant qu'un minuteur tourne, le mode affiché est celui du minuteur en cours.
  const mode: Mode = active ? (snapshot.kind === "pomodoro" ? "pomodoro" : "countdown") : chosenMode;

  const run = async (action: () => Promise<unknown>) => {
    try {
      await action();
      setError(null);
    } catch (reason) {
      setError(String(reason));
    }
  };

  const start = () =>
    run(() => (mode === "pomodoro" ? ipc.timerStartPomodoro() : ipc.timerStartCountdown(minutes * 60)));

  const idleSeconds = mode === "pomodoro" ? settings.timer.focusMinutes * 60 : minutes * 60;
  const shownSeconds = active ? snapshot.remainingSeconds : idleSeconds;
  const percent = active && snapshot.totalSeconds > 0 ? (snapshot.remainingSeconds / snapshot.totalSeconds) * 100 : 100;
  const color = phaseColor(active ? snapshot.phase : "focus");

  return (
    <BentoCard title="Timer" icon={<Timer size={14} />}>
      <div className="flex items-center gap-8">
        <ProgressRing percent={percent} color={color}>
          <span className="font-mono text-3xl font-semibold">{formatClock(shownSeconds)}</span>
          <span className="mt-1 text-xs text-white/55">
            {active && snapshot.phase ? PHASE_LABEL[snapshot.phase] : mode === "pomodoro" ? "Focus" : "Timer"}
          </span>
          {mode === "pomodoro" && (
            <div className="mt-2 flex gap-1.5" aria-label="Focus sessions in this cycle">
              {Array.from({ length: settings.timer.sessionsBeforeLongBreak }, (_, index) => (
                <span
                  key={index}
                  className="h-1.5 w-1.5 rounded-full"
                  style={{
                    backgroundColor: index < snapshot.cycleProgress ? "var(--accent-color)" : "rgba(255,255,255,0.2)",
                  }}
                />
              ))}
            </div>
          )}
        </ProgressRing>

        <div className="flex min-w-0 flex-1 flex-col gap-4">
          <div className="flex w-fit gap-1 rounded-xl bg-white/5 p-1">
            {(["pomodoro", "countdown"] as const).map((item) => (
              <button
                key={item}
                type="button"
                disabled={active}
                onClick={() => setChosenMode(item)}
                className={`rounded-lg px-3 py-1 text-xs font-medium transition-colors disabled:cursor-not-allowed ${
                  mode === item ? "bg-white/15 text-white" : "text-white/50 hover:text-white"
                } ${active && mode !== item ? "opacity-40" : ""}`}
              >
                {item === "pomodoro" ? "Pomodoro" : "Timer"}
              </button>
            ))}
          </div>

          {!active && mode === "pomodoro" && (
            <p className="text-xs leading-relaxed text-white/55">
              {settings.timer.focusMinutes} min of focus, {settings.timer.shortBreakMinutes} min break,
              then a {settings.timer.longBreakMinutes} min long break every{" "}
              {settings.timer.sessionsBeforeLongBreak} sessions. Durations are set in Settings.
            </p>
          )}

          {!active && mode === "countdown" && (
            <div className="flex flex-col gap-2">
              <div className="flex flex-wrap gap-1.5">
                {PRESET_MINUTES.map((preset) => (
                  <button
                    key={preset}
                    type="button"
                    onClick={() => setMinutes(preset)}
                    className={`rounded-full px-3 py-1 text-xs transition-colors ${
                      minutes === preset ? "bg-[color:var(--accent-color)]/30 text-white" : "bg-white/8 text-white/60 hover:text-white"
                    }`}
                  >
                    {preset} min
                  </button>
                ))}
              </div>
              <label className="flex items-center gap-2 text-xs text-white/55">
                Custom duration
                <input
                  type="number"
                  min={1}
                  max={MAX_MINUTES}
                  value={minutes}
                  onChange={(event) =>
                    setMinutes(Math.min(MAX_MINUTES, Math.max(1, Math.floor(Number(event.target.value)) || 1)))
                  }
                  className="w-16 rounded-lg border border-white/10 bg-white/5 px-2 py-1 text-center font-mono text-xs text-white outline-none focus:border-[color:var(--accent-color)]"
                />
                min
              </label>
            </div>
          )}

          {finished && (
            <p className="text-xs font-medium" style={{ color: phaseColor(finished.nextPhase ?? finished.phase) }}>
              {PHASE_LABEL[finished.phase]} finished
              {finished.nextPhase ? `, next up: ${PHASE_LABEL[finished.nextPhase].toLowerCase()}.` : "."}
            </p>
          )}

          <div className="flex items-center gap-2">
            {!active && (
              <button
                type="button"
                onClick={() => void start()}
                className="flex items-center gap-1.5 rounded-xl bg-[color:var(--accent-color)] px-4 py-2 text-sm font-medium text-black transition-transform hover:scale-[1.03]"
              >
                <Play size={14} className="fill-current" />
                Start
              </button>
            )}

            {active && snapshot.status === "running" && (
              <button
                type="button"
                onClick={() => void run(ipc.timerPause)}
                className="flex items-center gap-1.5 rounded-xl bg-white px-4 py-2 text-sm font-medium text-black transition-transform hover:scale-[1.03]"
              >
                <Pause size={14} className="fill-current" />
                Pause
              </button>
            )}

            {active && (snapshot.status === "paused" || snapshot.status === "ready") && (
              <button
                type="button"
                onClick={() => void run(ipc.timerResume)}
                className="flex items-center gap-1.5 rounded-xl bg-[color:var(--accent-color)] px-4 py-2 text-sm font-medium text-black transition-transform hover:scale-[1.03]"
              >
                <Play size={14} className="fill-current" />
                {snapshot.status === "ready" ? "Start phase" : "Resume"}
              </button>
            )}

            {active && snapshot.kind === "pomodoro" && (
              <button
                type="button"
                onClick={() => void run(ipc.timerSkip)}
                className="flex items-center gap-1.5 rounded-xl bg-white/8 px-3 py-2 text-xs text-white/70 transition-colors hover:bg-white/12 hover:text-white"
              >
                <SkipForward size={13} />
                Skip
              </button>
            )}

            {active && (
              <button
                type="button"
                onClick={() => void run(ipc.timerReset)}
                className="flex items-center gap-1.5 rounded-xl bg-white/8 px-3 py-2 text-xs text-white/70 transition-colors hover:bg-white/12 hover:text-white"
              >
                <RotateCcw size={13} />
                Reset
              </button>
            )}
          </div>
        </div>
      </div>
      {error && <p className="truncate text-xs text-red-400">{error}</p>}
    </BentoCard>
  );
}
