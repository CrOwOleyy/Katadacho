import { Volume2, VolumeX } from "lucide-react";
import { BentoCard } from "../../components/ui/BentoCard";
import { Slider } from "../../components/ui/Slider";
import type { AudioState } from "../../lib/types/controls";
import { AudioDevicePicker } from "./AudioDevicePicker";

interface VolumeCardProps {
  state: AudioState;
  onVolume: (level: number) => void;
  onToggleMute: () => void;
  onSelectDevice: (deviceId: string) => void;
}

export function VolumeCard({ state, onVolume, onToggleMute, onSelectDevice }: VolumeCardProps) {
  return (
    <BentoCard title="Volume" icon={state.muted ? <VolumeX size={14} /> : <Volume2 size={14} />} className="col-span-6">
      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={onToggleMute}
          aria-label={state.muted ? "Unmute" : "Mute"}
          className="rounded-lg p-1 text-white/70 transition-colors hover:bg-white/10 hover:text-white"
        >
          {state.muted ? <VolumeX size={16} /> : <Volume2 size={16} />}
        </button>
        <Slider
          value={state.volume}
          label="Main volume"
          onCommit={onVolume}
        />
        <span className="w-9 text-right font-mono text-xs text-white/60">{state.volume}</span>
      </div>
      <AudioDevicePicker
        devices={state.devices}
        activeId={state.activeDevice?.id}
        onSelect={onSelectDevice}
      />
    </BentoCard>
  );
}
