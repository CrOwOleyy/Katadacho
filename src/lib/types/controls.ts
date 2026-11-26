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

/** Volume d'une application dans le mixeur. */
export interface AppVolume {
  /** Identifiant stable, renvoyé tel quel pour régler l'application. */
  id: string;
  name: string;
  volume: number;
  muted: boolean;
  /** Fichier dont le système tire l'icône de l'application, quand elle en a une. */
  iconPath: string | null;
}
