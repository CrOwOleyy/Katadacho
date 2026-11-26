import { useCallback, useEffect, useState } from "react";
import type { GhostMood } from "../../components/mascot/mood";
import { ipc } from "../../lib/ipc";
import type { AppVolume, AudioState, NetworkBluetoothState } from "../../lib/types/controls";
import { BluetoothCard } from "./BluetoothCard";
import { BrightnessCard } from "./BrightnessCard";
import { MascotCard } from "./MascotCard";
import { MediaCard } from "./MediaCard";
import { MixerCard } from "./MixerCard";
import { VolumeCard } from "./VolumeCard";
import { WifiCard } from "./WifiCard";

const REFRESH_MS = 2000;

/** Dernières valeurs connues, conservées entre deux ouvertures du menu pour éviter un écran de chargement. */
const cache: {
  audio: AudioState | null;
  network: NetworkBluetoothState | null;
  brightness: number | null;
  apps: AppVolume[] | null;
} = { audio: null, network: null, brightness: null, apps: null };

export function ControlsTab({ mood }: { mood: GhostMood }) {
  const [audio, setAudio] = useState<AudioState | null>(cache.audio);
  const [network, setNetwork] = useState<NetworkBluetoothState | null>(cache.network);
  const [brightness, setBrightness] = useState<number | null>(cache.brightness);
  const [apps, setApps] = useState<AppVolume[] | null>(cache.apps);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const [audioResult, networkResult, brightnessResult, appsResult] = await Promise.allSettled([
      ipc.getAudioState(),
      ipc.getNetworkBtState(),
      ipc.getBrightness(),
      ipc.getAppVolumes(),
    ]);

    if (audioResult.status === "fulfilled") {
      cache.audio = audioResult.value;
      setAudio(audioResult.value);
    }
    if (networkResult.status === "fulfilled") {
      cache.network = networkResult.value;
      setNetwork(networkResult.value);
    }
    // Pas d'écran réglable : la carte luminosité l'affiche, ce n'est pas une erreur globale.
    cache.brightness = brightnessResult.status === "fulfilled" ? brightnessResult.value : null;
    setBrightness(cache.brightness);
    // Mixeur indisponible : la carte est simplement masquée, la carte Volume signale déjà le problème.
    cache.apps = appsResult.status === "fulfilled" ? appsResult.value : null;
    setApps(cache.apps);

    const failures = [audioResult, networkResult]
      .filter((result): result is PromiseRejectedResult => result.status === "rejected")
      .map((result) => String(result.reason));
    setError(failures.length > 0 ? failures.join(" | ") : null);
  }, []);

  useEffect(() => {
    void refresh();
    const interval = window.setInterval(() => void refresh(), REFRESH_MS);
    return () => window.clearInterval(interval);
  }, [refresh]);

  const run = useCallback(
    async (action: () => Promise<unknown>) => {
      try {
        await action();
      } catch (reason) {
        setError(String(reason));
      }
      await refresh();
    },
    [refresh],
  );

  if (!audio || !network) {
    return <p className="text-sm text-white/50">Loading system controls…</p>;
  }

  return (
    <div className="flex flex-col gap-3">
      <div className="grid grid-cols-6 gap-3">
        <MediaCard />
        <MascotCard mood={mood} />
        <WifiCard
          state={network}
          onToggle={(enable) => void run(() => ipc.toggleWifi(enable))}
        />
        <BluetoothCard
          state={network}
          onToggle={(enable) => void run(() => ipc.toggleBluetooth(enable))}
        />
        <BrightnessCard
          level={brightness}
          onCommit={(level) => void run(() => ipc.setBrightness(level))}
        />
        <VolumeCard
          state={audio}
          onVolume={(level) => void run(() => ipc.setMasterVolume(level))}
          onToggleMute={() => void run(() => ipc.toggleMute())}
          onSelectDevice={(id) => void run(() => ipc.setDefaultAudioDevice(id))}
        />
        {apps && (
          <MixerCard
            apps={apps}
            onVolume={(id, level) => void run(() => ipc.setAppVolume(id, level))}
            onToggleMute={(id) => void run(() => ipc.toggleAppMute(id))}
          />
        )}
      </div>
      {error && <p className="truncate text-xs text-red-400">{error}</p>}
    </div>
  );
}
