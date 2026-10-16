import { listen } from "@tauri-apps/api/event";
import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { ipc } from "../lib/ipc";
import type { Settings } from "../lib/types/settings";

const SAVE_DEBOUNCE_MS = 250;

/** Valeurs utilisées le temps que Rust réponde : identiques aux défauts côté Rust. */
const FALLBACK: Settings = {
  animations: { enabled: true, speed: 1, openDelayMs: 300, closeDelayMs: 450, tabTransitionMs: 140 },
  island: { autoHideSeconds: 15, showMediaInPill: true },
  timer: {
    focusMinutes: 25,
    shortBreakMinutes: 5,
    longBreakMinutes: 15,
    sessionsBeforeLongBreak: 4,
    autoStartNext: false,
    soundEnabled: true,
  },
  notifications: { enabled: true, displaySeconds: 6, showBody: true, soundEnabled: false, blockedApps: [] },
  clipboard: { detectSecrets: true, secretTtlMinutes: 2 },
};

interface SettingsContextValue {
  settings: Settings;
  /** Applique tout de suite à l'écran, puis enregistre après une courte pause. */
  update: (change: (current: Settings) => Settings) => void;
  reset: () => Promise<void>;
  error: string | null;
}

const SettingsContext = createContext<SettingsContextValue | null>(null);

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Settings>(FALLBACK);
  const [error, setError] = useState<string | null>(null);
  const saveTimer = useRef<number | undefined>(undefined);
  const latest = useRef<Settings>(FALLBACK);
  /** Numéro de la dernière modification locale, pour savoir si une sauvegarde est encore en attente. */
  const version = useRef(0);
  const savedVersion = useRef(0);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let active = true;

    ipc
      .getSettings()
      .then((loaded) => {
        if (!active) return;
        latest.current = loaded;
        setSettings(loaded);
      })
      .catch((reason: unknown) => setError(String(reason)));

    // Un autre composant ou un autre module Rust peut changer les réglages.
    listen<Settings>("settings-changed", (event) => {
      // Une modification locale non enregistrée est plus récente que cet événement : on l'ignore.
      if (savedVersion.current !== version.current) return;
      latest.current = event.payload;
      setSettings(event.payload);
    })
      .then((fn) => {
        if (active) unlisten = fn;
        else fn();
      })
      .catch((reason: unknown) => setError(String(reason)));

    return () => {
      active = false;
      unlisten?.();
      window.clearTimeout(saveTimer.current);
    };
  }, []);

  const update = useCallback((change: (current: Settings) => Settings) => {
    const next = change(latest.current);
    latest.current = next;
    setSettings(next);

    const myVersion = ++version.current;
    window.clearTimeout(saveTimer.current);
    saveTimer.current = window.setTimeout(() => {
      ipc
        .saveSettings(latest.current)
        .then(() => setError(null))
        .catch((reason: unknown) => setError(String(reason)))
        .finally(() => {
          // Seule la sauvegarde de la modification la plus récente lève la garde.
          if (version.current === myVersion) savedVersion.current = myVersion;
        });
    }, SAVE_DEBOUNCE_MS);
  }, []);

  const reset = useCallback(async () => {
    try {
      const defaults = await ipc.resetSettings();
      latest.current = defaults;
      setSettings(defaults);
      setError(null);
    } catch (reason) {
      setError(String(reason));
    }
  }, []);

  const value = useMemo(() => ({ settings, update, reset, error }), [settings, update, reset, error]);

  return <SettingsContext.Provider value={value}>{children}</SettingsContext.Provider>;
}

export function useSettings(): SettingsContextValue {
  const context = useContext(SettingsContext);
  if (!context) throw new Error("useSettings doit être utilisé dans un SettingsProvider");
  return context;
}
