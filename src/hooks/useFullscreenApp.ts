import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

const POLL_MS = 5000;

/** Vrai quand l'application au premier plan occupe tout l'écran. Inactif si `enabled` est faux. */
export function useFullscreenApp(enabled: boolean): boolean {
  const [fullscreen, setFullscreen] = useState(false);

  useEffect(() => {
    if (!enabled) {
      setFullscreen(false);
      return;
    }
    let active = true;
    const poll = () => {
      invoke<boolean>("is_fullscreen_app_active")
        .then((value) => {
          if (active) setFullscreen(value);
        })
        .catch(() => {
          if (active) setFullscreen(false);
        });
    };
    poll();
    const id = window.setInterval(poll, POLL_MS);
    return () => {
      active = false;
      window.clearInterval(id);
    };
  }, [enabled]);

  return fullscreen;
}
