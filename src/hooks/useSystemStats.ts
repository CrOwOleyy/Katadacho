import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { ipc } from "../lib/ipc";
import type { SystemStats } from "../lib/types/telemetry";

const HISTORY_LENGTH = 60;

export interface StatsHistory {
  cpu: number[];
  memory: number[];
  gpu: number[];
  disk: number[];
}

const append = (values: number[], value: number) => [...values, value].slice(-HISTORY_LENGTH);

/**
 * Démarre la collecte Rust tant que le composant est monté, puis la coupe.
 * Garde aussi 60 échantillons par métrique pour les sparklines.
 * `error` contient le message si l'écoute ou le démarrage échoue.
 */
export function useSystemStats() {
  const [stats, setStats] = useState<SystemStats | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [history, setHistory] = useState<StatsHistory>({ cpu: [], memory: [], gpu: [], disk: [] });

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let active = true;

    listen<SystemStats>("system-stats", (event) => {
      if (!active) return;
      const next = event.payload;
      setStats(next);
      setHistory((prev) => ({
        cpu: append(prev.cpu, next.cpu.usage),
        memory: append(prev.memory, next.memory.percent),
        gpu: append(prev.gpu, next.gpu?.usage ?? 0),
        disk: append(prev.disk, next.disk.totalGb > 0 ? (next.disk.usedGb / next.disk.totalGb) * 100 : 0),
      }));
    })
      .then((fn) => {
        // Si le composant est déjà démonté, on désinscrit tout de suite.
        if (active) {
          unlisten = fn;
        } else {
          fn();
        }
      })
      .catch((reason: unknown) => {
        if (active) setError(`Écoute des métriques impossible : ${String(reason)}`);
      });

    ipc.startSystemStats().catch((reason: unknown) => {
      if (active) setError(`Démarrage de la collecte impossible : ${String(reason)}`);
    });

    return () => {
      active = false;
      unlisten?.();
      ipc.stopSystemStats().catch((reason: unknown) => console.error("stop_system_stats failed", reason));
    };
  }, []);

  return { stats, history, error };
}
