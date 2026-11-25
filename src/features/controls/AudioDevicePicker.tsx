import { ChevronDown } from "lucide-react";
import type { AudioDevice } from "../../lib/types/controls";

interface AudioDevicePickerProps {
  devices: AudioDevice[];
  activeId: string | undefined;
  onSelect: (deviceId: string) => void;
}

export function AudioDevicePicker({ devices, activeId, onSelect }: AudioDevicePickerProps) {
  return (
    <div className="relative">
      <select
        aria-label="Audio output device"
        value={activeId ?? ""}
        disabled={devices.length === 0}
        onChange={(event) => onSelect(event.target.value)}
        className="w-full appearance-none truncate rounded-xl border border-white/10 bg-white/5 py-2 pl-3 pr-8 text-xs text-white outline-none transition-colors focus:border-[color:var(--accent-color)] disabled:opacity-40"
      >
        {devices.map((device) => (
          <option key={device.id} value={device.id} className="bg-[#121216]">
            {device.name}
          </option>
        ))}
      </select>
      <ChevronDown
        size={14}
        className="pointer-events-none absolute right-2.5 top-1/2 -translate-y-1/2 text-white/50"
      />
    </div>
  );
}
