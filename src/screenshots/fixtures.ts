// Fictional system data for the screenshot page. Nothing here comes from a real machine.

import type { AppVolume, AudioState, NetworkBluetoothState } from "../lib/types/controls";
import type { ClipboardItem } from "../lib/types/clipboard";
import type { LauncherResult } from "../lib/types/launcher";
import type { MediaInfo, MediaPosition } from "../lib/types/media";
import type { SystemStats } from "../lib/types/telemetry";
import type { TimerSnapshot } from "../lib/types/timer";

/** Draws a small PNG and returns it as base64, without the data URL prefix. */
function png(size: number, draw: (context: CanvasRenderingContext2D, size: number) => void): string {
  const canvas = document.createElement("canvas");
  canvas.width = size;
  canvas.height = size;
  const context = canvas.getContext("2d");
  if (context) draw(context, size);
  return canvas.toDataURL("image/png").split(",")[1];
}

/** Placeholder app icon: a rounded square in the app's colour with its initial. */
function appIcon(color: string, letter: string): string {
  return png(64, (context, size) => {
    context.fillStyle = color;
    context.beginPath();
    context.roundRect(4, 4, size - 8, size - 8, 14);
    context.fill();
    context.fillStyle = "#ffffff";
    context.font = "bold 30px sans-serif";
    context.textAlign = "center";
    context.textBaseline = "middle";
    context.fillText(letter, size / 2, size / 2 + 2);
  });
}

const cover = () =>
  png(256, (context, size) => {
    const gradient = context.createLinearGradient(0, 0, size, size);
    gradient.addColorStop(0, "#ff5f6d");
    gradient.addColorStop(0.5, "#7b2ff7");
    gradient.addColorStop(1, "#1fa2ff");
    context.fillStyle = gradient;
    context.fillRect(0, 0, size, size);
    context.fillStyle = "rgba(255,255,255,0.85)";
    context.beginPath();
    context.arc(size / 2, size * 0.62, size * 0.22, 0, Math.PI * 2);
    context.fill();
  });

export const ICONS: Record<string, () => string> = {
  "icon:spotify": () => appIcon("#1DB954", "S"),
  "icon:firefox": () => appIcon("#FF7139", "F"),
  "icon:discord": () => appIcon("#5865F2", "D"),
  "icon:system": () => appIcon("#64748b", "♪"),
  "app:firefox": () => appIcon("#FF7139", "F"),
  "app:figma": () => appIcon("#A259FF", "F"),
  "app:files": () => appIcon("#3b82f6", "F"),
};

export const media = (): MediaInfo => ({
  title: "Midnight Drive",
  artist: "Neon Coast",
  playing: true,
  coverBase64: cover(),
  coverMime: "image/png",
});

export const mediaPosition: MediaPosition = { positionSeconds: 83, durationSeconds: 214, playing: true };

export const audio: AudioState = {
  volume: 64,
  muted: false,
  activeDevice: { id: "speakers", name: "Speakers (Realtek Audio)", isDefault: true },
  devices: [
    { id: "speakers", name: "Speakers (Realtek Audio)", isDefault: true },
    { id: "headphones", name: "Headphones (Bluetooth)", isDefault: false },
  ],
};

export const network: NetworkBluetoothState = {
  wifiEnabled: true,
  wifiSsid: "Home-5G",
  wifiSignal: 82,
  bluetoothEnabled: true,
  bluetoothDevices: ["Headphones"],
};

export const appVolumes: AppVolume[] = [
  { id: "discord", name: "Discord", volume: 100, muted: true, iconPath: "icon:discord" },
  { id: "firefox", name: "Firefox", volume: 65, muted: false, iconPath: "icon:firefox" },
  { id: "spotify", name: "Spotify", volume: 80, muted: false, iconPath: "icon:spotify" },
  { id: "system", name: "System sounds", volume: 100, muted: false, iconPath: "icon:system" },
];

const minutesAgo = (minutes: number) => Date.now() - minutes * 60_000;

export const clipboard = (): ClipboardItem[] => [
  { id: 6, contentType: "url", textContent: "https://v2.tauri.app/start/", imagePath: null, thumbnailBase64: null, sourceAppName: "Firefox", isPinned: false, isSensitive: false, createdAt: minutesAgo(1) },
  { id: 5, contentType: "color", textContent: "#7C5CFF", imagePath: null, thumbnailBase64: null, sourceAppName: "Figma", isPinned: false, isSensitive: false, createdAt: minutesAgo(4) },
  { id: 4, contentType: "code", textContent: "cargo test --lib", imagePath: null, thumbnailBase64: null, sourceAppName: "Terminal", isPinned: true, isSensitive: false, createdAt: minutesAgo(9) },
  { id: 3, contentType: "image", textContent: null, imagePath: "capture.png", thumbnailBase64: cover(), sourceAppName: "Snipping Tool", isPinned: false, isSensitive: false, createdAt: minutesAgo(15) },
  { id: 2, contentType: "text", textContent: "sk-demo-0000000000000000", imagePath: null, thumbnailBase64: null, sourceAppName: "Terminal", isPinned: false, isSensitive: true, createdAt: minutesAgo(20) },
  { id: 1, contentType: "text", textContent: "Meeting moved to 3 pm, room B.", imagePath: null, thumbnailBase64: null, sourceAppName: "Discord", isPinned: false, isSensitive: false, createdAt: minutesAgo(32) },
];

export const launcherResults: LauncherResult[] = [
  { name: "Firefox", path: "app:firefox", kind: "app", score: 120 },
  { name: "Figma", path: "app:figma", kind: "app", score: 110 },
  { name: "Files", path: "app:files", kind: "app", score: 96 },
  { name: "Finances", path: "C:\\Users\\demo\\Documents\\Finances", kind: "folder", score: 80 },
  { name: "final-report.pdf", path: "C:\\Users\\demo\\Documents\\final-report.pdf", kind: "file", score: 72 },
];

export const idleTimer: TimerSnapshot = {
  kind: "none",
  phase: null,
  status: "idle",
  remainingSeconds: 0,
  totalSeconds: 0,
  completedFocusSessions: 0,
  sessionsBeforeLongBreak: 4,
  cycleProgress: 0,
};

export const focusTimer: TimerSnapshot = {
  kind: "pomodoro",
  phase: "focus",
  status: "running",
  remainingSeconds: 1122,
  totalSeconds: 1500,
  completedFocusSessions: 2,
  sessionsBeforeLongBreak: 4,
  cycleProgress: 0.6,
};

/** One sample of fake telemetry, gently varying with `tick` so the charts have a shape. */
export function stats(tick: number): SystemStats {
  const wave = (speed: number, phase = 0) => (Math.sin(tick * speed + phase) + 1) / 2;
  return {
    cpu: { usage: 18 + 22 * wave(0.45), frequencyGhz: 3.9, temperatureC: 52 + 6 * wave(0.3) },
    memory: { usedGb: 11.2 + wave(0.2), totalGb: 32, percent: 36 + 3 * wave(0.2), swapUsedGb: 0.4, swapTotalGb: 8 },
    disk: { usedGb: 612, totalGb: 1000, readMbS: 12 * wave(0.9, 1), writeMbS: 4 * wave(0.7, 2) },
    network: { downloadMbS: 3.5 + 3 * wave(0.6, 1), uploadMbS: 0.4 + 0.3 * wave(0.5) },
    gpu: { name: "Radeon RX 7800 XT", usage: 12 + 10 * wave(0.4, 3), vramUsedMb: 3100, vramTotalMb: 16384 },
    processes: [
      { name: "Firefox", pid: 4120, cpu: 7.8, memoryMb: 1840, count: 12 },
      { name: "Code", pid: 5310, cpu: 4.1, memoryMb: 1220, count: 9 },
      { name: "Discord", pid: 6022, cpu: 2.6, memoryMb: 640, count: 5 },
      { name: "Spotify", pid: 7011, cpu: 1.9, memoryMb: 410, count: 4 },
      { name: "Katadacho", pid: 8123, cpu: 0.8, memoryMb: 120, count: 2 },
    ],
  };
}
