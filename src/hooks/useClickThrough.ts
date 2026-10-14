import { useEffect, type RefObject } from "react";
import { ipc } from "../lib/ipc";

/**
 * Envoie au Rust le rectangle exact de l'élément suivi à chaque changement de taille
 * ou de position. Le thread natif s'en sert pour décider si la fenêtre capte les clics.
 */
export function useClickThroughBounds<T extends HTMLElement>(ref: RefObject<T | null>) {
  useEffect(() => {
    const element = ref.current;
    if (!element) return;

    let frame = 0;
    const push = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const rect = element.getBoundingClientRect();
        ipc
          .updateIslandBounds({
            x: rect.left,
            y: rect.top,
            width: rect.width,
            height: rect.height,
          })
          .catch((error: unknown) => console.error("update_island_bounds failed", error));
      });
    };

    const observer = new ResizeObserver(push);
    observer.observe(element);
    window.addEventListener("resize", push);
    push();

    return () => {
      cancelAnimationFrame(frame);
      observer.disconnect();
      window.removeEventListener("resize", push);
    };
  }, [ref]);
}
