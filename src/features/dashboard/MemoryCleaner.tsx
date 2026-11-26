import { MemoryStick, ShieldCheck, Sparkles } from "lucide-react";
import { useState } from "react";
import { BentoCard } from "../../components/ui/BentoCard";
import { ipc } from "../../lib/ipc";
import { pulseMood } from "../../lib/moodPulse";
import type { CleanResult } from "../../lib/types/memory";

type Running = "quick" | "deep" | null;

const BYTES_PER_GB = 1_000_000_000;

function formatGb(bytes: number): string {
  return (bytes / BYTES_PER_GB).toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function summary(result: CleanResult): string {
  const detail = result.mode === "quick" ? ` (${result.processesTrimmed} processes)` : "";
  if (result.freedBytes <= 0) {
    return `Nothing more to free, ${formatGb(result.usedAfterBytes)} GB in use${detail}`;
  }
  return `${formatGb(result.freedBytes)} GB freed${detail}, ${formatGb(result.usedAfterBytes)} GB in use`;
}

/** Libère de la RAM : rapide sans droits particuliers, ou profond avec les droits administrateur. */
export function MemoryCleaner() {
  const [running, setRunning] = useState<Running>(null);
  const [result, setResult] = useState<CleanResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const run = async (mode: Exclude<Running, null>) => {
    setRunning(mode);
    setError(null);
    try {
      const outcome = mode === "quick" ? await ipc.cleanMemoryQuick() : await ipc.cleanMemoryDeep();
      setResult(outcome);
      pulseMood("happy");
    } catch (reason) {
      setResult(null);
      pulseMood("sad");
      setError(String(reason));
    } finally {
      setRunning(null);
    }
  };

  return (
    <BentoCard title="Memory" icon={<MemoryStick size={14} />}>
      <div className="flex items-center justify-between gap-4">
        <div className="min-w-0 text-xs text-white/55">
          {running === "quick" && "Trimming app memory…"}
          {running === "deep" && "Deep clean running, accept the permission prompt…"}
          {running === null && result && <span className="text-white/80">{summary(result)}</span>}
          {running === null && !result && !error && "Gives memory back to the system without closing any app."}
          {running === null && error && <span className="text-red-400">{error}</span>}
        </div>

        <div className="flex shrink-0 items-center gap-2">
          <button
            type="button"
            disabled={running !== null}
            onClick={() => void run("quick")}
            className="flex items-center gap-1.5 rounded-xl bg-[color:var(--accent-color)]/20 px-3 py-1.5 text-xs font-medium transition-colors hover:bg-[color:var(--accent-color)]/30 disabled:opacity-40"
          >
            <Sparkles size={13} />
            Free RAM
          </button>
          <button
            type="button"
            disabled={running !== null}
            onClick={() => void run("deep")}
            title="Also empties the system caches. Asks for administrator permission."
            className="flex items-center gap-1.5 rounded-xl bg-white/8 px-3 py-1.5 text-xs font-medium text-white/80 transition-colors hover:bg-white/12 hover:text-white disabled:opacity-40"
          >
            <ShieldCheck size={13} />
            Deep clean
          </button>
        </div>
      </div>
      {result && result.failedSteps.length > 0 && (
        <p className="text-[10px] text-amber-300/80">Steps not completed: {result.failedSteps.join(", ")}</p>
      )}
    </BentoCard>
  );
}
