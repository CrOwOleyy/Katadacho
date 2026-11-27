/** Accent used when there is no cover, or when the cover has no usable color. */
export const DEFAULT_ACCENT = "#7c9cff";

/** Pixels paler, darker or greyer than this are ignored: they would give a washed-out accent. */
const MIN_SATURATION = 0.25;
const MIN_LIGHTNESS = 0.15;
const MAX_LIGHTNESS = 0.9;
/** Hue buckets of 15 degrees: wide enough to merge shades of one color, narrow enough to tell colors apart. */
const HUE_BUCKETS = 24;
/** Below this share of colorful pixels, the cover is considered grey and the default accent stays. */
const MIN_COLORFUL_SHARE = 0.03;
/** The accent sits on a near-black island: it is brightened and saturated until it reads well there. */
const TARGET_LIGHTNESS = 0.66;
const MIN_ACCENT_SATURATION = 0.55;

interface Hsl {
  h: number;
  s: number;
  l: number;
}

export function rgbToHsl(r: number, g: number, b: number): Hsl {
  const red = r / 255;
  const green = g / 255;
  const blue = b / 255;
  const max = Math.max(red, green, blue);
  const min = Math.min(red, green, blue);
  const l = (max + min) / 2;
  if (max === min) return { h: 0, s: 0, l };

  const delta = max - min;
  const s = l > 0.5 ? delta / (2 - max - min) : delta / (max + min);
  let h: number;
  if (max === red) h = (green - blue) / delta + (green < blue ? 6 : 0);
  else if (max === green) h = (blue - red) / delta + 2;
  else h = (red - green) / delta + 4;
  return { h: h * 60, s, l };
}

export function hslToHex({ h, s, l }: Hsl): string {
  const chroma = (1 - Math.abs(2 * l - 1)) * s;
  const x = chroma * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = l - chroma / 2;
  const [r, g, b] =
    h < 60 ? [chroma, x, 0]
    : h < 120 ? [x, chroma, 0]
    : h < 180 ? [0, chroma, x]
    : h < 240 ? [0, x, chroma]
    : h < 300 ? [x, 0, chroma]
    : [chroma, 0, x];
  const hex = (value: number) => Math.round((value + m) * 255).toString(16).padStart(2, "0");
  return `#${hex(r)}${hex(g)}${hex(b)}`;
}

/**
 * Picks the dominant vivid color of a cover from its RGBA pixels, adjusted to read well on the dark
 * island. Returns `null` when the cover is mostly grey, black or white.
 */
export function pickAccent(pixels: Uint8ClampedArray | number[]): string | null {
  const weight = new Array<number>(HUE_BUCKETS).fill(0);
  const hueX = new Array<number>(HUE_BUCKETS).fill(0);
  const hueY = new Array<number>(HUE_BUCKETS).fill(0);
  const saturation = new Array<number>(HUE_BUCKETS).fill(0);
  let total = 0;
  let colorful = 0;

  for (let index = 0; index + 3 < pixels.length; index += 4) {
    if (pixels[index + 3] < 128) continue;
    total += 1;
    const color = rgbToHsl(pixels[index], pixels[index + 1], pixels[index + 2]);
    if (color.s < MIN_SATURATION || color.l < MIN_LIGHTNESS || color.l > MAX_LIGHTNESS) continue;
    colorful += 1;

    const bucket = Math.floor(color.h / (360 / HUE_BUCKETS)) % HUE_BUCKETS;
    // Vivid pixels count more than dull ones, so a small bright logo can win over a large muted area.
    const strength = color.s * color.s;
    weight[bucket] += strength;
    // Hues are averaged as angles, so 350 and 10 degrees average to 0, not 180.
    const radians = (color.h * Math.PI) / 180;
    hueX[bucket] += Math.cos(radians) * strength;
    hueY[bucket] += Math.sin(radians) * strength;
    saturation[bucket] += color.s * strength;
  }

  if (total === 0 || colorful / total < MIN_COLORFUL_SHARE) return null;

  const best = weight.indexOf(Math.max(...weight));
  const hue = ((Math.atan2(hueY[best], hueX[best]) * 180) / Math.PI + 360) % 360;
  const averageSaturation = saturation[best] / weight[best];
  return hslToHex({ h: hue, s: Math.max(averageSaturation, MIN_ACCENT_SATURATION), l: TARGET_LIGHTNESS });
}
