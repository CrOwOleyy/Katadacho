import { Wifi } from "lucide-react";
import { BentoCard } from "../../components/ui/BentoCard";
import { Toggle } from "../../components/ui/Toggle";
import type { NetworkBluetoothState } from "../../lib/types/controls";

interface WifiCardProps {
  state: NetworkBluetoothState;
  onToggle: (enable: boolean) => void;
}

export function WifiCard({ state, onToggle }: WifiCardProps) {
  const connected = state.wifiSsid !== null;

  return (
    <BentoCard title="Wi-Fi" icon={<Wifi size={14} />} className="col-span-2">
      <div className="flex items-center justify-between">
        <span className="truncate text-sm font-medium">
          {connected ? state.wifiSsid : state.wifiEnabled === false ? "Désactivé" : "Non connecté"}
        </span>
        <Toggle
          checked={state.wifiEnabled}
          label="Activer le Wi-Fi"
          onChange={onToggle}
        />
      </div>
      {connected && state.wifiSignal !== null && (
        <div className="flex items-center gap-2">
          <div className="h-1 flex-1 overflow-hidden rounded-full bg-white/10">
            <div
              className="h-full rounded-full bg-[color:var(--accent-color)] transition-[width] duration-300"
              style={{ width: `${state.wifiSignal}%` }}
            />
          </div>
          <span className="font-mono text-xs text-white/60">{state.wifiSignal}%</span>
        </div>
      )}
    </BentoCard>
  );
}
