import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { isVideoApp, type ForegroundApp } from "../lib/videoApps";

const POLL_MS = 5000;

/** Vrai quand l'application au premier plan est un lecteur vidéo ou un site de streaming. Inactif si `enabled` est faux. */
export function useVideoApp(enabled: boolean): boolean {
  const [video, setVideo] = useState(false);

  useEffect(() => {
    if (!enabled) {
      setVideo(false);
      return;
    }
    let active = true;
    const poll = () => {
      invoke<ForegroundApp | null>("get_foreground_app")
        .then((app) => {
          if (active) setVideo(app !== null && isVideoApp(app));
        })
        .catch(() => {
          if (active) setVideo(false);
        });
    };
    poll();
    const id = window.setInterval(poll, POLL_MS);
    return () => {
      active = false;
      window.clearInterval(id);
    };
  }, [enabled]);

  return video;
}
