import { BentoCard } from "../../components/ui/BentoCard";
import { RingGauge } from "../../components/ui/RingGauge";
import { Sparkline } from "../../components/ui/Sparkline";
import { useSystemStats } from "../../hooks/useSystemStats";
import { ipc } from "../../lib/ipc";
import { Activity } from "lucide-react";
import { ProcessList } from "./ProcessList";
import { SpeedtestWidget } from "./SpeedtestWidget";

const formatGb = (value: number) => value.toFixed(1);

export function DashboardTab() {
  const { stats, history, error } = useSystemStats();

  if (!stats) {
    return error ? (
      <p className="text-sm text-red-400">{error}</p>
    ) : (
      <p className="text-sm text-white/50">Lecture des métriques…</p>
    );
  }

  const diskPercent = stats.disk.totalGb > 0 ? (stats.disk.usedGb / stats.disk.totalGb) * 100 : 0;

  return (
    <div className="flex flex-col gap-3">
      <div className="grid grid-cols-4 gap-2">
        <RingGauge
          value={stats.cpu.usage}
          label="CPU"
          sublabel={`${stats.cpu.frequencyGhz.toFixed(2)} GHz${stats.cpu.temperatureC !== null ? ` · ${Math.round(stats.cpu.temperatureC)}°C` : ""}`}
        />
        <RingGauge
          value={stats.memory.percent}
          label="RAM"
          sublabel={`${formatGb(stats.memory.usedGb)} / ${formatGb(stats.memory.totalGb)} Go`}
        />
        <RingGauge
          value={stats.gpu?.usage ?? 0}
          label="GPU"
          sublabel={
            stats.gpu
              ? `${formatGb(stats.gpu.vramUsedMb / 1024)} / ${formatGb(stats.gpu.vramTotalMb / 1024)} Go VRAM`
              : "Non détecté"
          }
        />
        <RingGauge
          value={diskPercent}
          label="Disque"
          sublabel={`${formatGb(stats.disk.usedGb)} / ${formatGb(stats.disk.totalGb)} Go`}
        />
      </div>

      <div className="grid grid-cols-4 gap-2">
        <Sparkline values={history.cpu} />
        <Sparkline values={history.memory} />
        <Sparkline values={history.gpu} />
        <Sparkline values={history.disk} />
      </div>

      <div className="grid grid-cols-2 gap-3">
        <BentoCard title="Réseau" icon={<Activity size={14} />}>
          <div className="grid grid-cols-2 gap-2 font-mono text-xs">
            <span className="text-white/60">↓ {stats.network.downloadMbS.toFixed(2)} Mo/s</span>
            <span className="text-white/60">↑ {stats.network.uploadMbS.toFixed(2)} Mo/s</span>
            <span className="text-white/60">Lecture disque {stats.disk.readMbS.toFixed(2)} Mo/s</span>
            <span className="text-white/60">Écriture disque {stats.disk.writeMbS.toFixed(2)} Mo/s</span>
          </div>
        </BentoCard>

        <BentoCard title="Top processus" icon={<Activity size={14} />}>
          <ProcessList processes={stats.processes} onKill={(pid) => ipc.killProcess(pid)} />
        </BentoCard>

        <SpeedtestWidget />
      </div>
    </div>
  );
}
