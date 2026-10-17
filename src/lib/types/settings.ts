export interface AnimationSettings {
  enabled: boolean;
  speed: number;
  openDelayMs: number;
  closeDelayMs: number;
  tabTransitionMs: number;
}

export interface IslandSettings {
  autoHideSeconds: number;
  showMediaInPill: boolean;
}

export interface TimerSettings {
  focusMinutes: number;
  shortBreakMinutes: number;
  longBreakMinutes: number;
  sessionsBeforeLongBreak: number;
  autoStartNext: boolean;
  soundEnabled: boolean;
}

export interface NotificationSettings {
  enabled: boolean;
  displaySeconds: number;
  showBody: boolean;
  soundEnabled: boolean;
  blockedApps: string[];
}

export interface ClipboardSettings {
  detectSecrets: boolean;
  secretTtlMinutes: number;
}

export type LauncherHotkey = "copilot" | "alt+space" | "ctrl+alt+space" | "ctrl+shift+space";

export interface HotkeySettings {
  launcher: LauncherHotkey;
}

export interface MascotSettings {
  happyOnTasks: boolean;
  focusOnFullscreen: boolean;
  sleepAtNight: boolean;
  nightStartHour: number;
  nightEndHour: number;
}

export interface Settings {
  mascot: MascotSettings;
  hotkeys: HotkeySettings;
  animations: AnimationSettings;
  island: IslandSettings;
  timer: TimerSettings;
  notifications: NotificationSettings;
  clipboard: ClipboardSettings;
}
