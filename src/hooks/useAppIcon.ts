import { useEffect, useState } from "react";
import { ipc } from "../lib/ipc";

/** Une icône introuvable est redemandée après ce délai : l'échec n'était peut-être que passager. */
export const RETRY_AFTER_MS = 30_000;

/** Icônes déjà obtenues, partagées par le lanceur et le mixeur. */
const icons = new Map<string, string>();
/** Moment du dernier échec, pour ne pas redemander en boucle une icône absente. */
const failedAt = new Map<string, number>();
/** Demandes en cours : deux composants qui veulent la même icône partagent une seule demande. */
const pending = new Map<string, Promise<string | null>>();

/** Icône PNG en base64, ou `null` si elle n'est pas disponible pour le moment. */
export function loadIcon(path: string): Promise<string | null> {
  const known = icons.get(path);
  if (known) return Promise.resolve(known);

  const failed = failedAt.get(path);
  if (failed !== undefined && Date.now() - failed < RETRY_AFTER_MS) return Promise.resolve(null);

  let request = pending.get(path);
  if (!request) {
    request = ipc
      .getLauncherIcon(path)
      .then((base64) => (base64 ? base64 : null))
      .catch(() => null)
      .then((icon) => {
        if (icon) {
          icons.set(path, icon);
          failedAt.delete(path);
        } else {
          failedAt.set(path, Date.now());
        }
        return icon;
      })
      .finally(() => pending.delete(path));
    pending.set(path, request);
  }
  return request;
}

/** Vide les caches. Réservé aux tests. */
export function resetIconCache() {
  icons.clear();
  failedAt.clear();
  pending.clear();
}

/**
 * Icône PNG en base64 d'une application ou d'un fichier, extraite par le système (exécutable sous
 * Windows, thème d'icônes sous Linux). `null` tant qu'elle charge, ou quand il n'y en a pas ; tant
 * que le composant reste affiché, une icône manquante est redemandée de temps en temps.
 */
export function useAppIcon(path: string | null | undefined): string | null {
  const [icon, setIcon] = useState<string | null>(path ? (icons.get(path) ?? null) : null);

  useEffect(() => {
    if (!path) {
      setIcon(null);
      return;
    }
    let active = true;
    let retry: number | undefined;
    const attempt = () => {
      void loadIcon(path).then((result) => {
        if (!active) return;
        setIcon(result);
        if (result === null) retry = window.setTimeout(attempt, RETRY_AFTER_MS);
      });
    };
    attempt();
    return () => {
      active = false;
      window.clearTimeout(retry);
    };
  }, [path]);

  return icon;
}
