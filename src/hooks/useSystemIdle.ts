import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

/** Interrogation espacée quand l'utilisateur est actif, rapprochée quand il est inactif pour réveiller vite. */
const ACTIVE_POLL_MS = 10_000;
const IDLE_POLL_MS = 2_000;

/**
 * Vrai quand personne n'a touché le clavier ni la souris depuis `thresholdSeconds`, sur tout le système.
 * Si la mesure n'est pas disponible (autre système), reste à `false`.
 */
export function useSystemIdle(thresholdSeconds: number): boolean {
  const [idle, setIdle] = useState(false);

  useEffect(() => {
    let active = true;
    let timer = 0;

    const poll = async () => {
      let nowIdle = false;
      try {
        const seconds = await invoke<number>("get_idle_seconds");
        nowIdle = seconds >= thresholdSeconds;
      } catch {
        nowIdle = false;
      }
      if (!active) return;
      setIdle(nowIdle);
      timer = window.setTimeout(() => void poll(), nowIdle ? IDLE_POLL_MS : ACTIVE_POLL_MS);
    };

    void poll();
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [thresholdSeconds]);

  return idle;
}
