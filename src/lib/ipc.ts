import { Channel, invoke } from "@tauri-apps/api/core";
import type { AudioState, IslandBounds, NetworkBluetoothState } from "./types/controls";
import type { SpeedtestEvent } from "./types/speedtest";
import type { ClipboardFilter, ClipboardItem } from "./types/clipboard";
import type { LauncherResult } from "./types/launcher";
import type { MediaInfo, MediaPosition } from "./types/media";
import type { CleanResult } from "./types/memory";
import type { Settings } from "./types/settings";
import type { TimerSnapshot } from "./types/timer";
import type { NotificationAccess } from "./types/notifications";

export const ipc = {
  updateIslandBounds: (bounds: IslandBounds) => invoke<void>("update_island_bounds", { bounds }),

  getAudioState: () => invoke<AudioState>("get_audio_state"),
  setMasterVolume: (level: number) => invoke<void>("set_master_volume", { level }),
  toggleMute: () => invoke<boolean>("toggle_mute"),
  setDefaultAudioDevice: (deviceId: string) =>
    invoke<void>("set_default_audio_device", { deviceId }),

  getBrightness: () => invoke<number>("get_brightness"),
  setBrightness: (level: number) => invoke<void>("set_brightness", { level }),

  getNetworkBtState: () => invoke<NetworkBluetoothState>("get_network_bt_state"),
  toggleWifi: (enable: boolean) => invoke<void>("toggle_wifi", { enable }),
  toggleBluetooth: (enable: boolean) => invoke<void>("toggle_bluetooth", { enable }),

  cleanMemoryQuick: () => invoke<CleanResult>("clean_memory_quick"),
  cleanMemoryDeep: () => invoke<CleanResult>("clean_memory_deep"),

  startSystemStats: () => invoke<void>("start_system_stats"),
  stopSystemStats: () => invoke<void>("stop_system_stats"),
  killProcess: (pid: number) => invoke<void>("kill_process", { pid }),

  getClipboardHistory: (searchQuery: string, filter: ClipboardFilter, limit = 200) =>
    invoke<ClipboardItem[]>("get_clipboard_history", { searchQuery, filter, limit }),
  togglePinClipboard: (id: number) => invoke<boolean>("toggle_pin_clipboard", { id }),
  deleteClipboardItem: (id: number) => invoke<void>("delete_clipboard_item", { id }),
  pasteClipboardItem: (id: number, simulatePaste: boolean) =>
    invoke<void>("paste_clipboard_item", { id, simulatePaste }),

  searchLauncher: (query: string) => invoke<LauncherResult[]>("search_launcher", { query }),
  launchItem: (path: string, runAsAdmin: boolean) => invoke<void>("launch_item", { path, runAsAdmin }),
  openItemLocation: (path: string) => invoke<void>("open_item_location", { path }),
  reindexLauncher: () => invoke<void>("reindex_launcher"),
  getLauncherIcon: (path: string) => invoke<string>("get_launcher_icon", { path }),

  getMediaState: () => invoke<MediaInfo | null>("get_media_state"),
  mediaTogglePlayPause: () => invoke<void>("media_toggle_play_pause"),
  mediaNext: () => invoke<void>("media_next"),
  mediaPrevious: () => invoke<void>("media_previous"),
  mediaSeek: (seconds: number) => invoke<void>("media_seek", { seconds }),
  mediaPosition: () => invoke<MediaPosition | null>("media_position"),
  startAudioLevels: () => invoke<void>("start_audio_levels"),
  stopAudioLevels: () => invoke<void>("stop_audio_levels"),

  timerGet: () => invoke<TimerSnapshot>("timer_get"),
  timerStartPomodoro: () => invoke<TimerSnapshot>("timer_start_pomodoro"),
  timerStartCountdown: (seconds: number) => invoke<TimerSnapshot>("timer_start_countdown", { seconds }),
  timerPause: () => invoke<TimerSnapshot>("timer_pause"),
  timerResume: () => invoke<TimerSnapshot>("timer_resume"),
  timerReset: () => invoke<TimerSnapshot>("timer_reset"),
  timerSkip: () => invoke<TimerSnapshot>("timer_skip"),

  getNotificationAccess: () => invoke<NotificationAccess>("get_notification_access"),
  requestNotificationAccess: () => invoke<NotificationAccess>("request_notification_access"),

  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  resetSettings: () => invoke<Settings>("reset_settings"),

  runSpeedtest: (onEvent: (event: SpeedtestEvent) => void) => {
    const channel = new Channel<SpeedtestEvent>();
    channel.onmessage = onEvent;
    return invoke<void>("run_speedtest", { channel });
  },
};
