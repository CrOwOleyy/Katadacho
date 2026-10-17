import { MemoryStick, ShieldCheck, Sparkles } from "lucide-react";
import { useState } from "react";
import { BentoCard } from "../../components/ui/BentoCard";
import { ipc } from "../../lib/ipc";
import { pulseMood } from "../../lib/moodPulse";
import type { CleanResult } from "../../lib/types/memory";

type Running = "quick" | "deep" | null;

const BYTES_PER_GB = 1_000_000_000;

function formatGb(bytes: number): string {
  return (bytes / BYTES_PER_GB).toLocaleString("fr-FR", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function summary(result: CleanResult): string {
  const detail = result.mode === "quick" ? ` (${result.processesTrimmed} processus)` : "";
  if (result.freedBytes <= 0) {
    return `Rien de plus à libérer, ${formatGb(result.usedAfterBytes)} Go utilisés${detail}`;
  }
  return `${formatGb(result.freedBytes)} Go libérés${detail}, ${formatGb(result.usedAfterBytes)} Go utilisés`;
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
    <BentoCard title="Mémoire" icon={<MemoryStick size={14} />}>
      <div className="flex items-center justify-between gap-4">
        <div className="min-w-0 text-xs text-white/55">
          {running === "quick" && "Réduction de la mémoire des applications…"}
          {running === "deep" && "Nettoyage profond en cours, accepte la demande d'autorisation Windows…"}
          {running === null && result && <span className="text-white/80">{summary(result)}</span>}
          {running === null && !result && !error && "Rend de la mémoire à Windows sans fermer aucune application."}
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
            Libérer la RAM
          </button>
          <button
            type="button"
            disabled={running !== null}
            onClick={() => void run("deep")}
            title="Vide aussi les caches système. Windows demande l'autorisation administrateur."
            className="flex items-center gap-1.5 rounded-xl bg-white/8 px-3 py-1.5 text-xs font-medium text-white/80 transition-colors hover:bg-white/12 hover:text-white disabled:opacity-40"
          >
            <ShieldCheck size={13} />
            Nettoyage profond
          </button>
        </div>
      </div>
      {result && result.failedSteps.length > 0 && (
        <p className="text-[10px] text-amber-300/80">Étapes non réalisées : {result.failedSteps.join(", ")}</p>
      )}
    </BentoCard>
  );
}
