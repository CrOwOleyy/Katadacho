import { Search } from "lucide-react";
import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { ipc } from "../../lib/ipc";
import { pulseMood } from "../../lib/moodPulse";
import type { LauncherResult } from "../../lib/types/launcher";
import { ResultRow } from "./ResultRow";

interface LauncherTabProps {
  /** Change à chaque ouverture par raccourci, pour redonner le focus au champ de recherche. */
  focusToken: number;
  /** Appelé après un lancement réussi, pour refermer l'Island. */
  onDone: () => void;
}

export function LauncherTab({ focusToken, onDone }: LauncherTabProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<LauncherResult[]>([]);
  const [selected, setSelected] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const requestCounter = useRef(0);

  useEffect(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  }, [focusToken]);

  // La recherche répond en quelques millisecondes : on l'exécute à chaque frappe.
  // Le compteur ignore les réponses arrivées dans le désordre.
  useEffect(() => {
    const request = ++requestCounter.current;
    ipc
      .searchLauncher(query)
      .then((found) => {
        if (request !== requestCounter.current) return;
        setResults(found);
        setSelected(0);
      })
      .catch((reason: unknown) => setError(String(reason)));
  }, [query]);

  const launch = async (result: LauncherResult, asAdmin: boolean) => {
    try {
      await ipc.launchItem(result.path, asAdmin);
      setError(null);
      setQuery("");
      onDone();
    } catch (reason) {
      setError(String(reason));
      pulseMood("sad");
    }
  };

  const openLocation = async (result: LauncherResult) => {
    try {
      await ipc.openItemLocation(result.path);
      setError(null);
    } catch (reason) {
      setError(String(reason));
    }
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    const current = results[selected];
    switch (event.key) {
      case "ArrowDown":
        event.preventDefault();
        setSelected((index) => Math.min(index + 1, results.length - 1));
        break;
      case "ArrowUp":
        event.preventDefault();
        setSelected((index) => Math.max(index - 1, 0));
        break;
      case "Enter":
        if (current) void launch(current, event.ctrlKey);
        break;
      case "Tab":
        event.preventDefault();
        if (current) void openLocation(current);
        break;
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="relative">
        <Search size={15} className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-white/40" />
        <input
          ref={inputRef}
          autoFocus
          type="text"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          onKeyDown={onKeyDown}
          placeholder="Lancer une application ou chercher un fichier"
          className="w-full rounded-xl border border-white/10 bg-white/5 py-2.5 pl-9 pr-3 text-sm outline-none placeholder:text-white/35 focus:border-[color:var(--accent-color)]"
        />
      </div>

      {error && <p className="truncate text-xs text-red-400">{error}</p>}

      <div className="flex max-h-[290px] flex-col gap-0.5 overflow-y-auto [scrollbar-width:none]">
        {results.map((result, index) => (
          <ResultRow
            key={`${result.kind}:${result.path}`}
            result={result}
            selected={index === selected}
            onActivate={() => void launch(result, false)}
            onHover={() => setSelected(index)}
          />
        ))}
        {query.trim() !== "" && results.length === 0 && (
          <p className="py-4 text-center text-sm text-white/45">Aucun résultat</p>
        )}
      </div>

      <p className="text-[10px] text-white/35">
        ↑ ↓ naviguer · Entrée lancer · Ctrl+Entrée administrateur · Tab ouvrir l'emplacement
      </p>
    </div>
  );
}
