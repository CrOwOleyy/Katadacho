import { describe, expect, it } from "vitest";
import { hslToHex, pickAccent, rgbToHsl } from "./coverAccent";

/** RGBA pixels: `count` copies of one color. */
function fill(count: number, r: number, g: number, b: number, a = 255): number[] {
  return Array.from({ length: count }, () => [r, g, b, a]).flat();
}

function hueOf(hex: string): number {
  const value = Number.parseInt(hex.slice(1), 16);
  return rgbToHsl((value >> 16) & 255, (value >> 8) & 255, value & 255).h;
}

describe("color conversions", () => {
  it("round trips through HSL", () => {
    const { h, s, l } = rgbToHsl(29, 185, 84);
    expect(hslToHex({ h, s, l })).toBe("#1db954");
  });
});

describe("pickAccent", () => {
  it("returns the dominant vivid hue", () => {
    const accent = pickAccent([...fill(80, 220, 30, 30), ...fill(20, 30, 30, 220)]);
    expect(accent).not.toBeNull();
    expect(Math.min(hueOf(accent!), 360 - hueOf(accent!))).toBeLessThan(10);
  });

  it("prefers a small vivid area over a large grey one", () => {
    const accent = pickAccent([...fill(90, 120, 120, 125), ...fill(10, 30, 200, 90)]);
    expect(hueOf(accent!)).toBeGreaterThan(130);
    expect(hueOf(accent!)).toBeLessThan(160);
  });

  it("gives up on a grey, black or white cover", () => {
    expect(pickAccent([...fill(50, 10, 10, 10), ...fill(50, 245, 245, 245)])).toBeNull();
    expect(pickAccent(fill(100, 128, 128, 128))).toBeNull();
  });

  it("ignores transparent pixels", () => {
    expect(pickAccent(fill(100, 255, 0, 0, 0))).toBeNull();
  });

  it("brightens a dark color so it reads on the black island", () => {
    const accent = pickAccent(fill(100, 60, 10, 90))!;
    const value = Number.parseInt(accent.slice(1), 16);
    const { l } = rgbToHsl((value >> 16) & 255, (value >> 8) & 255, value & 255);
    expect(l).toBeGreaterThan(0.6);
  });

  it("averages hues across the red wrap-around", () => {
    const accent = pickAccent([...fill(50, 255, 0, 20), ...fill(50, 255, 20, 0)])!;
    expect(Math.min(hueOf(accent), 360 - hueOf(accent))).toBeLessThan(8);
  });
});
