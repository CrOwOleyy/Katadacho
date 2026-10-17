import { useEffect, useRef, useState } from "react";
import type { GhostMood } from "../components/mascot/mood";
import { onMoodPulse } from "../lib/moodPulse";

/** Humeur ponctuelle demandée par l'interface, ou `null` quand elle est terminée. */
export function useMoodPulse(): GhostMood | null {
  const [pulse, setPulse] = useState<GhostMood | null>(null);
  const timer = useRef<number | undefined>(undefined);

  useEffect(() => {
    const unsubscribe = onMoodPulse(({ mood, durationMs }) => {
      setPulse(mood);
      window.clearTimeout(timer.current);
      timer.current = window.setTimeout(() => setPulse(null), durationMs);
    });
    return () => {
      unsubscribe();
      window.clearTimeout(timer.current);
    };
  }, []);

  return pulse;
}
