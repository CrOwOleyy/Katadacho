// Screenshot page: renders the real island with fictional data, for the README.
// Run `pnpm dev`, then open /screenshots.html?scene=controls (see SCENES). `body[data-ready]` is set
// once the scene has settled, so a headless browser knows when to take the picture.

import { emit } from "@tauri-apps/api/event";
import { mockIPC } from "@tauri-apps/api/mocks";
import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource-variable/inter";
import "@fontsource-variable/jetbrains-mono";
import "../styles/globals.css";
import { IslandApp } from "../island/IslandApp";
import type { MediaInfo, MediaPosition } from "../lib/types/media";
import { FALLBACK, SettingsProvider } from "../store/SettingsContext";
import * as fixtures from "./fixtures";

/** Each scene opens the island on one tab; `pill` keeps it collapsed, `mixer` scrolls Controls down. */
const SCENES = ["pill", "controls", "mixer", "dashboard", "clipboard", "launcher", "timer", "settings"] as const;
type Scene = (typeof SCENES)[number];

const requested = new URLSearchParams(window.location.search).get("scene");
const scene: Scene = SCENES.includes(requested as Scene) ? (requested as Scene) : "controls";

const settings = { ...FALLBACK, island: { ...FALLBACK.island, autoHideSeconds: 0 } };
const timer = scene === "timer" ? fixtures.focusTimer : fixtures.idleTimer;
const timers: number[] = [];

type NowPlaying = MediaInfo & Partial<Omit<MediaPosition, "playing">>;

/**
 * Optional real track for the media screenshots, in public/screenshot-media.json (ignored by git):
 * { title, artist, coverBase64, coverMime, positionSeconds, durationSeconds }. Without it, the page
 * uses the fictional track.
 */
let nowPlaying: NowPlaying | null = null;

async function loadNowPlaying(): Promise<NowPlaying | null> {
  try {
    const response = await fetch("/screenshot-media.json", { cache: "no-store" });
    if (!response.ok) return null;
    const data = (await response.json()) as NowPlaying;
    return data.title ? { ...data, playing: true } : null;
  } catch {
    // The dev server answers unknown paths with the HTML page: not JSON, so no real track.
    return null;
  }
}

function every(milliseconds: number, action: (tick: number) => void) {
  let tick = 0;
  action(tick);
  timers.push(window.setInterval(() => action(++tick), milliseconds));
}

mockIPC(
  (command, args) => {
    const payload = (args ?? {}) as Record<string, unknown>;
    switch (command) {
      case "get_settings":
        return settings;
      case "get_media_state":
        return nowPlaying ?? fixtures.media();
      case "media_position":
        return nowPlaying?.positionSeconds !== undefined
          ? { positionSeconds: nowPlaying.positionSeconds, durationSeconds: nowPlaying.durationSeconds ?? null, playing: true }
          : fixtures.mediaPosition;
      case "start_audio_levels":
        every(60, (tick) => void emit("audio-level", 0.35 + 0.3 * Math.abs(Math.sin(tick * 0.7))));
        return null;
      case "get_audio_state":
        return fixtures.audio;
      case "get_network_bt_state":
        return fixtures.network;
      case "get_brightness":
        return 72;
      case "get_app_volumes":
        return fixtures.appVolumes;
      case "get_launcher_icon": {
        const draw = fixtures.ICONS[String(payload.path)];
        if (!draw) throw new Error("no icon");
        return draw();
      }
      case "start_system_stats":
        every(400, (tick) => void emit("system-stats", fixtures.stats(tick)));
        return null;
      case "get_clipboard_history":
        return fixtures.clipboard();
      case "search_launcher":
        return String(payload.query ?? "").trim() ? fixtures.launcherResults : [];
      case "timer_get":
        return timer;
      case "get_hotkey_status":
        return { choice: settings.hotkeys.launcher, active: true, error: null };
      case "get_claude_bridge_status":
        return [{ path: "claude_desktop_config.json", exists: true, installed: true, upToDate: true }];
      case "get_notification_access":
        return "allowed";
      case "get_idle_seconds":
        return 0;
      case "is_fullscreen_app_active":
      case "plugin:autostart|is_enabled":
        return false;
      default:
        return null;
    }
  },
  { shouldMockEvents: true },
);


const wait = (milliseconds: number) => new Promise((resolve) => window.setTimeout(resolve, milliseconds));

function clickButton(label: string) {
  const button = [...document.querySelectorAll("button")].find((element) => element.textContent?.trim() === label);
  button?.click();
}

/** Types into a React-controlled input, as a user would. */
function type(input: HTMLInputElement, text: string) {
  const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
  setter?.call(input, text);
  input.dispatchEvent(new Event("input", { bubbles: true }));
}

const TAB_LABELS: Partial<Record<Scene, string>> = {
  dashboard: "Dashboard",
  clipboard: "Clipboard",
  launcher: "Launcher",
  timer: "Timer",
  settings: "Settings",
};

async function play() {
  await wait(800);
  if (scene !== "pill") {
    // The pill is the first element inside the island root: a click opens the menu.
    (document.querySelector("#root > div > div") as HTMLElement | null)?.click();
    await wait(700);
    const tab = TAB_LABELS[scene];
    if (tab) clickButton(tab);
    await wait(500);
    if (scene === "launcher") {
      const input = document.querySelector("input[type='text'], input:not([type])") as HTMLInputElement | null;
      if (input) type(input, "fi");
    }
    if (scene === "mixer") {
      // The tab content scrolls: bring the mixer, at the bottom of Controls, into view.
      const scroller = [...document.querySelectorAll<HTMLElement>("*")].find(
        (element) => element.scrollHeight > element.clientHeight + 20 && getComputedStyle(element).overflowY === "auto",
      );
      if (scroller) scroller.scrollTop = scroller.scrollHeight;
    }
  }
  // Long enough for charts to fill and springs to settle.
  await wait(scene === "dashboard" ? 6000 : 1500);
  document.body.dataset.ready = "true";
}

async function start() {
  nowPlaying = await loadNowPlaying();
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <SettingsProvider>
        <IslandApp />
      </SettingsProvider>
    </React.StrictMode>,
  );
  await play();
}

void start();
