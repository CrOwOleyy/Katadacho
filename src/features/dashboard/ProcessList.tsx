import { useState } from "react";
import type { ProcessGroup } from "../../lib/types/telemetry";

interface ProcessListProps {
  processes: ProcessGroup[];
  onKill: (pid: number) => Promise<void>;
}

/** Top des processus regroupés par nom. Le bouton Kill apparaît au survol de la ligne. */
export function ProcessList({ processes, onKill }: ProcessListProps) {
  const [error, setError] = useState<string | null>(null);

  const kill = async (pid: number) => {
    try {
      await onKill(pid);
      setError(null);
    } catch (reason) {
      setError(String(reason));
    }
  };

  return (
    <div className="flex flex-col gap-1">
      {processes.map((process) => (
        <div
          key={process.name}
          className="group flex items-center gap-2 rounded-lg px-2 py-1 text-xs transition-colors hover:bg-white/5"
        >
          <span className="min-w-0 flex-1 truncate">
            {process.name}
            {process.count > 1 && <span className="ml-1 text-white/40">×{process.count}</span>}
          </span>
          <span className="w-12 text-right font-mono text-white/60">{process.cpu.toFixed(1)}%</span>
          <span className="w-16 text-right font-mono text-white/60">{Math.round(process.memoryMb)} MB</span>
          <button
            type="button"
            onClick={() => void kill(process.pid)}
            className="rounded-md bg-red-500/20 px-2 py-0.5 text-[10px] font-medium text-red-300 opacity-0 transition-opacity group-hover:opacity-100 hover:bg-red-500/30"
          >
            Kill
          </button>
        </div>
      ))}
      {error && <p className="truncate text-[10px] text-red-400">{error}</p>}
    </div>
  );
}
