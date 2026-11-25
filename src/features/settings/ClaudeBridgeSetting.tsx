import { Bot } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { BentoCard } from "../../components/ui/BentoCard";
import { ipc } from "../../lib/ipc";
import type { ClaudeConfigStatus } from "../../lib/types/ai";

type Summary = "not-found" | "not-installed" | "stale" | "installed";

function summarize(statuses: ClaudeConfigStatus[]): Summary {
  const existing = statuses.filter((status) => status.exists);
  if (existing.length === 0) return "not-found";
  if (existing.every((status) => status.installed && status.upToDate)) return "installed";
  if (existing.some((status) => status.installed)) return "stale";
  return "not-installed";
}

const LABEL: Record<Summary, string> = {
  "not-found": "Claude Desktop not found",
  "not-installed": "Not connected",
  stale: "Connected, update needed",
  installed: "Connected",
};

/** Branche ou débranche Katadacho de Claude Desktop, sans clé d'API, par un serveur MCP local. */
export function ClaudeBridgeSetting() {
  const [statuses, setStatuses] = useState<ClaudeConfigStatus[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setStatuses(await ipc.getClaudeBridgeStatus());
    } catch (reason) {
      setError(String(reason));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const run = async (action: () => Promise<string[]>, done: string) => {
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      await action();
      setMessage(done);
      await refresh();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  };

  const summary = statuses ? summarize(statuses) : null;
  const connected = summary === "installed" || summary === "stale";

  return (
    <BentoCard title="Claude Desktop" icon={<Bot size={14} />}>
      <p className="text-[11px] leading-snug text-white/45">
        Claude can send you notifications and ask you questions in the island, with no API key.
      </p>
      <div className="flex items-center justify-between gap-3">
        <span
          className={`rounded-full px-2.5 py-0.5 text-[11px] font-medium ${
            summary === "installed" ? "bg-emerald-500/20 text-emerald-300" : "bg-white/10 text-white/60"
          }`}
        >
          {summary ? LABEL[summary] : "…"}
        </span>
        <div className="flex gap-2">
          {summary !== "not-found" && (
            <button
              type="button"
              disabled={busy}
              onClick={() => void run(ipc.installClaudeMcpBridge, "Connected. Restart Claude Desktop to turn it on.")}
              className="rounded-lg bg-[color:var(--accent-color)]/25 px-3 py-1 text-xs font-medium hover:bg-[color:var(--accent-color)]/35 disabled:opacity-50"
            >
              {summary === "stale" ? "Update" : connected ? "Reconnect" : "Connect"}
            </button>
          )}
          {connected && (
            <button
              type="button"
              disabled={busy}
              onClick={() => void run(ipc.uninstallClaudeMcpBridge, "Disconnected. Restart Claude Desktop.")}
              className="rounded-lg bg-white/8 px-3 py-1 text-xs text-white/70 hover:bg-white/12 hover:text-white disabled:opacity-50"
            >
              Disconnect
            </button>
          )}
        </div>
      </div>
      {message && <p className="text-[11px] text-emerald-300/80">{message}</p>}
      {error && <p className="text-[11px] text-red-400">{error}</p>}
      {summary === "not-found" && (
        <p className="text-[11px] text-white/45">Open Claude Desktop at least once, then come back here.</p>
      )}
    </BentoCard>
  );
}
