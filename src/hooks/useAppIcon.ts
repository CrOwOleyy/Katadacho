import { useEffect, useState } from "react";
import { ipc } from "../lib/ipc";

/** Cache mémoire des icônes déjà demandées, partagé par le lanceur et le mixeur. */
const iconCache = new Map<string, string | null>();

/**
 * Icône PNG en base64 d'une application ou d'un fichier, extraite par le système (exécutable sous
 * Windows, thème d'icônes sous Linux). `null` tant qu'elle charge, ou quand il n'y en a pas.
 */
export function useAppIcon(path: string | null | undefined): string | null {
  const [icon, setIcon] = useState<string | null>(path ? (iconCache.get(path) ?? null) : null);

  useEffect(() => {
    if (!path) {
      setIcon(null);
      return;
    }
    if (iconCache.has(path)) {
      setIcon(iconCache.get(path) ?? null);
      return;
    }
    let active = true;
    ipc
      .getLauncherIcon(path)
      .then((base64) => {
        iconCache.set(path, base64);
        if (active) setIcon(base64);
      })
      .catch(() => {
        // Pas d'icône disponible : on mémorise l'échec et on garde l'icône de repli.
        iconCache.set(path, null);
        if (active) setIcon(null);
      });
    return () => {
      active = false;
    };
  }, [path]);

  return icon;
}
