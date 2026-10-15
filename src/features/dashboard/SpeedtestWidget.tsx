import { useState } from "react";
import { BentoCard } from "../../components/ui/BentoCard";
import { ipc } from "../../lib/ipc";
import type { SpeedtestEvent } from "../../lib/types/speedtest";
import { SpeedGauge } from "./SpeedGauge";
import { Gauge } from "lucide-react";

type Phase = "idle" | "ping" | "download" | "upload" | "done" | "error";

interface Result {
  pingMs: number | null;
  jitterMs: number | null;
  downloadMbps: number | null;
  uploadMbps: number | null;
}

const EMPTY: Result = { pingMs: null, jitterMs: null, downloadMbps: null, uploadMbps: null };

export function SpeedtestWidget() {
  const [phase, setPhase] = useState<Phase>("idle");
  const [live, setLive] = useState(0);
  const [result, setResult] = useState<Result>(EMPTY);
  const [error, setError] = useState<string | null>(null);

  const run = async () => {
    setPhase("ping");
    setLive(0);
    setResult(EMPTY);
    setError(null);

    const onEvent = (event: SpeedtestEvent) => {
      switch (event.stage) {
        case "ping":
          setResult((prev) => ({ ...prev, pingMs: event.latencyMs, jitterMs: event.jitterMs }));
          setPhase("download");
          break;
        case "download":
          setPhase("download");
          setLive(event.mbps);
          break;
        case "upload":
          setPhase("upload");
          setLive(event.mbps);
          break;
        case "done":
          setResult((prev) => ({
            ...prev,
            downloadMbps: event.downloadMbps,
            uploadMbps: event.uploadMbps,
          }));
          setLive(0);
          setPhase("done");
          break;
      }
    };

    try {
      await ipc.runSpeedtest(onEvent);
    } catch (reason) {
      setError(String(reason));
      setPhase("error");
    }
  };

  const running = phase === "ping" || phase === "download" || phase === "upload";

  return (
    <BentoCard title="Speedtest" icon={<Gauge size={14} />} className="col-span-4">
      <div className="flex flex-col items-center gap-3">
        <SpeedGauge
          mbps={live}
          label={phase === "upload" ? "Envoi" : phase === "download" ? "Réception" : "Mbps"}
        />
        <div className="grid w-full grid-cols-2 gap-2 text-xs">
          <Stat label="Ping" value={result.pingMs} unit="ms" />
          <Stat label="Gigue" value={result.jitterMs} unit="ms" />
          <Stat label="Download" value={result.downloadMbps} unit="Mbps" />
          <Stat label="Upload" value={result.uploadMbps} unit="Mbps" />
        </div>
      </div>
      <button
        type="button"
        disabled={running}
        onClick={() => void run()}
        className="self-start rounded-lg bg-[color:var(--accent-color)]/20 px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-[color:var(--accent-color)]/30 disabled:opacity-40"
      >
        {running ? "Test en cours…" : "Lancer le speedtest"}
      </button>
      {error && <p className="truncate text-[10px] text-red-400">{error}</p>}
    </BentoCard>
  );
}

function Stat({ label, value, unit }: { label: string; value: number | null; unit: string }) {
  return (
    <div className="flex flex-col rounded-lg bg-white/5 px-2 py-1.5">
      <span className="text-[10px] uppercase tracking-wider text-white/45">{label}</span>
      <span className="font-mono text-sm">
        {value === null ? "—" : value.toFixed(1)}
        {value !== null && <span className="ml-1 text-[10px] text-white/45">{unit}</span>}
      </span>
    </div>
  );
}
