import { listen } from "@tauri-apps/api/event";
import { Search } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { ipc } from "../../lib/ipc";
import type { ClipboardFilter, ClipboardItem as Item } from "../../lib/types/clipboard";
import { useSettings } from "../../store/SettingsContext";
import { ClipboardItem } from "./ClipboardItem";
import { TypeFilter } from "./TypeFilter";

const SEARCH_DEBOUNCE_MS = 120;
const COPIED_FLASH_MS = 1200;

export function ClipboardTab() {
  const { settings } = useSettings();
  const [items, setItems] = useState<Item[]>([]);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<ClipboardFilter>("all");
  const [copiedId, setCopiedId] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loaded, setLoaded] = useState(false);
  const copiedTimer = useRef<number | undefined>(undefined);

  const load = useCallback(async () => {
    try {
      setItems(await ipc.getClipboardHistory(query, filter));
      setError(null);
    } catch (reason) {
      setError(String(reason));
    } finally {
      setLoaded(true);
    }
  }, [query, filter]);

  // Recherche instantanée avec un court délai de frappe.
  useEffect(() => {
    const timeout = window.setTimeout(() => void load(), SEARCH_DEBOUNCE_MS);
    return () => window.clearTimeout(timeout);
  }, [load]);

  // Chaque nouvelle copie, n'importe où dans Windows, rafraîchit la liste.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let active = true;
    listen("clipboard-updated", () => void load())
      .then((fn) => {
        if (active) unlisten = fn;
        else fn();
      })
      .catch((reason: unknown) => setError(`Écoute du presse-papier impossible : ${String(reason)}`));
    return () => {
      active = false;
      unlisten?.();
    };
  }, [load]);

  useEffect(() => () => window.clearTimeout(copiedTimer.current), []);

  const run = async (action: () => Promise<unknown>) => {
    try {
      await action();
      setError(null);
    } catch (reason) {
      setError(String(reason));
    }
  };

  const copy = (item: Item) =>
    run(async () => {
      await ipc.pasteClipboardItem(item.id, false);
      setCopiedId(item.id);
      window.clearTimeout(copiedTimer.current);
      copiedTimer.current = window.setTimeout(() => setCopiedId(null), COPIED_FLASH_MS);
    });

  return (
    <div className="flex flex-col gap-3">
      <div className="relative">
        <Search size={14} className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-white/40" />
        <input
          type="text"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Rechercher dans l'historique"
          className="w-full rounded-xl border border-white/10 bg-white/5 py-2 pl-9 pr-3 text-sm outline-none placeholder:text-white/35 focus:border-[color:var(--accent-color)]"
        />
      </div>

      <TypeFilter value={filter} onChange={setFilter} />

      {error && <p className="truncate text-xs text-red-400">{error}</p>}

      <div className="flex flex-col gap-1.5">
        {items.map((item) => (
          <ClipboardItem
            key={item.id}
            item={item}
            copied={copiedId === item.id}
            secretTtlMinutes={settings.clipboard.secretTtlMinutes}
            onCopy={() => void copy(item)}
            onPaste={() => void run(() => ipc.pasteClipboardItem(item.id, true))}
            onTogglePin={() => void run(async () => { await ipc.togglePinClipboard(item.id); await load(); })}
            onDelete={() => void run(async () => { await ipc.deleteClipboardItem(item.id); await load(); })}
          />
        ))}
        {loaded && items.length === 0 && !error && (
          <p className="py-6 text-center text-sm text-white/45">
            {query || filter !== "all" ? "Aucun résultat" : "Copie quelque chose avec Ctrl+C, il apparaîtra ici"}
          </p>
        )}
      </div>
    </div>
  );
}
