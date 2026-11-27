import { useEffect } from "react";
import { DEFAULT_ACCENT, pickAccent } from "../lib/coverAccent";
import type { MediaInfo } from "../lib/types/media";

/** The cover is shrunk before reading its pixels: plenty to find a color, and nearly free. */
const SAMPLE_SIZE = 48;

function applyAccent(color: string) {
  document.documentElement.style.setProperty("--accent-color", color);
}

/** Dominant color of an image, or `null` when it has none or cannot be read. */
function readAccent(source: string): Promise<string | null> {
  return new Promise((resolve) => {
    const image = new Image();
    image.onload = () => {
      const canvas = document.createElement("canvas");
      canvas.width = SAMPLE_SIZE;
      canvas.height = SAMPLE_SIZE;
      const context = canvas.getContext("2d", { willReadFrequently: true });
      if (!context) return resolve(null);
      context.drawImage(image, 0, 0, SAMPLE_SIZE, SAMPLE_SIZE);
      resolve(pickAccent(context.getImageData(0, 0, SAMPLE_SIZE, SAMPLE_SIZE).data));
    };
    image.onerror = () => resolve(null);
    image.src = source;
  });
}

/**
 * Tints the interface (sliders, rings, buttons) with the main color of the playing track's cover.
 * Without a cover, with a grey one, or when the setting is off, the default accent comes back.
 */
export function useCoverAccent(media: MediaInfo | null, enabled: boolean) {
  const cover = media?.coverBase64 ? `data:${media.coverMime ?? "image/jpeg"};base64,${media.coverBase64}` : null;

  useEffect(() => {
    if (!enabled || !cover) {
      applyAccent(DEFAULT_ACCENT);
      return;
    }
    let active = true;
    void readAccent(cover).then((color) => {
      if (active) applyAccent(color ?? DEFAULT_ACCENT);
    });
    return () => {
      active = false;
    };
  }, [cover, enabled]);
}
