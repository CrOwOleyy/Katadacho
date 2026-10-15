import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import type { NotificationToast } from "../lib/types/notifications";
import { useSettings } from "../store/SettingsContext";

interface Active extends NotificationToast {
  expiresAt: number;
}

/** Plafond de la file : au-delà, les plus anciennes sont oubliées. */
const MAX_QUEUE = 20;

/**
 * Notifications Windows à afficher dans l'Island. La plus récente est affichée, les autres sont
 * comptées. Chacune disparaît après la durée réglée.
 */
export function useNotifications() {
  const { settings } = useSettings();
  const { enabled, displaySeconds } = settings.notifications;
  const [queue, setQueue] = useState<Active[]>([]);
  const [now, setNow] = useState(() => Date.now());
  const displayMs = useRef(displaySeconds * 1000);
  const isEnabled = useRef(enabled);

  displayMs.current = displaySeconds * 1000;
  isEnabled.current = enabled;

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let active = true;

    listen<NotificationToast>("notification-arrived", (event) => {
      if (!isEnabled.current) return;
      const arrival = Date.now();
      setNow(arrival);
      setQueue((previous) =>
        [...previous, { ...event.payload, expiresAt: arrival + displayMs.current }].slice(-MAX_QUEUE),
      );
    })
      .then((fn) => {
        if (active) unlisten = fn;
        else fn();
      })
      .catch((reason: unknown) => console.error("notification listener failed", reason));

    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  // Retire les notifications expirées, en se réveillant juste à la prochaine échéance.
  useEffect(() => {
    if (queue.length === 0) return;
    const nextExpiry = Math.min(...queue.map((toast) => toast.expiresAt));
    const timeout = window.setTimeout(() => {
      const current = Date.now();
      setNow(current);
      setQueue((previous) => previous.filter((toast) => toast.expiresAt > current));
    }, Math.max(0, nextExpiry - Date.now()) + 20);
    return () => window.clearTimeout(timeout);
  }, [queue]);

  // La désactivation de la fonction vide immédiatement la file.
  useEffect(() => {
    if (!enabled) setQueue([]);
  }, [enabled]);

  const dismissCurrent = useCallback(() => {
    setQueue((previous) => previous.slice(0, -1));
  }, []);

  const visibleQueue = queue.filter((toast) => toast.expiresAt > now);
  const current = visibleQueue.length > 0 ? visibleQueue[visibleQueue.length - 1] : null;

  return { current, pending: Math.max(0, visibleQueue.length - 1), dismissCurrent };
}
