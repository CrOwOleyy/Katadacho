import { motion } from "motion/react";
import { Coffee, Timer } from "lucide-react";
import type { TimerSnapshot } from "../../lib/types/timer";
import { formatClock, phaseColor } from "./format";

interface TimerChipProps {
  snapshot: TimerSnapshot;
  /** Vrai pendant quelques secondes après la fin d'une phase : la puce pulse. */
  flashing: boolean;
}

/** Puce du minuteur dans la pilule repliée : icône de phase et temps restant. */
export function TimerChip({ snapshot, flashing }: TimerChipProps) {
  const isBreak = snapshot.phase === "shortBreak" || snapshot.phase === "longBreak";
  const Icon = isBreak ? Coffee : Timer;
  const dimmed = snapshot.status === "paused";

  return (
    <motion.div
      animate={flashing ? { scale: [1, 1.15, 1] } : { scale: 1 }}
      transition={flashing ? { duration: 0.9, repeat: Infinity } : { duration: 0.15 }}
      className={`flex shrink-0 items-center gap-1 rounded-full bg-white/10 px-2 py-0.5 ${dimmed ? "opacity-50" : ""}`}
      style={{ color: phaseColor(snapshot.phase) }}
    >
      <Icon size={12} />
      <span className="font-mono text-[11px] text-white">
        {snapshot.status === "ready" ? "Ready" : formatClock(snapshot.remainingSeconds)}
      </span>
    </motion.div>
  );
}
