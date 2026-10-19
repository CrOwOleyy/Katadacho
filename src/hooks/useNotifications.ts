import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { pulseMood } from "../lib/moodPulse";
import type { AiNotification } from "../lib/types/ai";
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
    const unlisteners: Array<() => void> = [];
    let active = true;
    let aiCounter = 0;

    const push = (toast: NotificationToast) => {
      const arrival = Date.now();
      setNow(arrival);
      setQueue((previous) => [...previous, { ...toast, expiresAt: arrival + displayMs.current }].slice(-MAX_QUEUE));
    };

    const subscribe = <T,>(event: string, handler: (payload: T) => void) => {
      listen<T>(event, (e) => handler(e.payload))
        .then((fn) => {
          if (active) unlisteners.push(fn);
          else fn();
        })
        .catch((reason: unknown) => console.error(`${event} listener failed`, reason));
    };

    subscribe<NotificationToast>("notification-arrived", (toast) => {
      if (isEnabled.current) push(toast);
    });

    // Messages envoyés par Claude Desktop via le pont MCP : demandés explicitement, donc affichés
    // même si les notifications Windows sont coupées. Les identifiants négatifs évitent tout conflit.
    subscribe<AiNotification>("ai-notify", (message) => {
      aiCounter += 1;
      push({ id: -aiCounter, appName: "Claude", title: message.title, body: message.summary, createdMs: Date.now() });
      if (message.status === "done") pulseMood("happy");
      if (message.status === "error") pulseMood("sad");
    });

    return () => {
      active = false;
      unlisteners.forEach((fn) => fn());
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
