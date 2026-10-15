import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { ipc } from "../lib/ipc";
import type { MediaInfo } from "../lib/types/media";

/** Média en cours de lecture dans Windows (Spotify, navigateur, etc.), ou `null` s'il n'y en a pas. */
export function useMedia(): MediaInfo | null {
  const [media, setMedia] = useState<MediaInfo | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let active = true;

    ipc
      .getMediaState()
      .then((state) => {
        if (active) setMedia(state);
      })
      .catch((reason: unknown) => console.error("get_media_state failed", reason));

    listen<MediaInfo | null>("media-changed", (event) => setMedia(event.payload))
      .then((fn) => {
        if (active) unlisten = fn;
        else fn();
      })
      .catch((reason: unknown) => console.error("media-changed listener failed", reason));

    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  return media;
}
