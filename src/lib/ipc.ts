import { Channel, invoke } from "@tauri-apps/api/core";
import type { AudioState, IslandBounds, NetworkBluetoothState } from "./types/controls";
import type { SpeedtestEvent } from "./types/speedtest";

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

  startSystemStats: () => invoke<void>("start_system_stats"),
  stopSystemStats: () => invoke<void>("stop_system_stats"),
  killProcess: (pid: number) => invoke<void>("kill_process", { pid }),

  runSpeedtest: (onEvent: (event: SpeedtestEvent) => void) => {
    const channel = new Channel<SpeedtestEvent>();
    channel.onmessage = onEvent;
    return invoke<void>("run_speedtest", { channel });
  },
};
