export interface AudioDevice {
  id: string;
  name: string;
  isDefault: boolean;
}

export interface AudioState {
  volume: number;
  muted: boolean;
  activeDevice: AudioDevice | null;
  devices: AudioDevice[];
}

export interface NetworkBluetoothState {
  wifiEnabled: boolean | null;
  wifiSsid: string | null;
  wifiSignal: number | null;
  bluetoothEnabled: boolean | null;
  bluetoothDevices: string[];
}

export interface IslandBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}
