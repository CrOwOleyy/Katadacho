import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { ipc } from "../lib/ipc";
import type { TimerFinished, TimerSnapshot } from "../lib/types/timer";

const IDLE: TimerSnapshot = {
  kind: "none",
  phase: null,
  status: "idle",
  remainingSeconds: 0,
  totalSeconds: 0,
  completedFocusSessions: 0,
  sessionsBeforeLongBreak: 4,
  cycleProgress: 0,
};

/** Durée pendant laquelle la pilule signale visuellement une fin de phase. */
const FINISHED_FLASH_MS = 6000;

/**
 * État du minuteur, tenu par Rust : il continue de tourner même si l'Island est cachée.
 * `finished` vaut le dernier événement de fin de phase pendant quelques secondes, pour l'animation.
 */
export function useTimer() {
  const [snapshot, setSnapshot] = useState<TimerSnapshot>(IDLE);
  const [finished, setFinished] = useState<TimerFinished | null>(null);

  useEffect(() => {
    let active = true;
    const unlisteners: Array<() => void> = [];
    let flashTimer: number | undefined;

    ipc
      .timerGet()
      .then((current) => {
        if (active) setSnapshot(current);
      })
      .catch((reason: unknown) => console.error("timer_get failed", reason));

    const subscribe = <T,>(event: string, handler: (payload: T) => void) => {
      listen<T>(event, (e) => handler(e.payload))
        .then((fn) => {
          if (active) unlisteners.push(fn);
          else fn();
        })
        .catch((reason: unknown) => console.error(`${event} listener failed`, reason));
    };

    subscribe<TimerSnapshot>("timer-update", setSnapshot);
    subscribe<TimerFinished>("timer-finished", (event) => {
      setFinished(event);
      window.clearTimeout(flashTimer);
      flashTimer = window.setTimeout(() => setFinished(null), FINISHED_FLASH_MS);
    });

    return () => {
      active = false;
      unlisteners.forEach((fn) => fn());
      window.clearTimeout(flashTimer);
    };
  }, []);

  return { snapshot, finished };
}
