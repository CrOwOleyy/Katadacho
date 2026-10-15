import { Bluetooth } from "lucide-react";
import { BentoCard } from "../../components/ui/BentoCard";
import { Toggle } from "../../components/ui/Toggle";
import type { NetworkBluetoothState } from "../../lib/types/controls";

interface BluetoothCardProps {
  state: NetworkBluetoothState;
  onToggle: (enable: boolean) => void;
}

export function BluetoothCard({ state, onToggle }: BluetoothCardProps) {
  const [first, ...others] = state.bluetoothDevices;

  return (
    <BentoCard title="Bluetooth" icon={<Bluetooth size={14} />} className="col-span-2">
      <div className="flex items-center justify-between">
        <span className="truncate text-sm font-medium">
          {first
            ? others.length > 0
              ? `${first} +${others.length}`
              : first
            : state.bluetoothEnabled === false
              ? "Désactivé"
              : "Aucun appareil"}
        </span>
        <Toggle
          checked={state.bluetoothEnabled}
          label="Activer le Bluetooth"
          onChange={onToggle}
        />
      </div>
    </BentoCard>
  );
}
