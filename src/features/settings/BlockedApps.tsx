import { X } from "lucide-react";
import { useState } from "react";

interface BlockedAppsProps {
  apps: string[];
  onChange: (apps: string[]) => void;
}

/** Liste de noms d'applications dont les notifications sont ignorées. */
export function BlockedApps({ apps, onChange }: BlockedAppsProps) {
  const [draft, setDraft] = useState("");

  const add = () => {
    const name = draft.trim();
    if (!name || apps.some((app) => app.toLowerCase() === name.toLowerCase())) {
      setDraft("");
      return;
    }
    onChange([...apps, name]);
    setDraft("");
  };

  return (
    <div className="flex flex-col gap-2">
      <div className="flex gap-2">
        <input
          type="text"
          value={draft}
          onChange={(event) => setDraft(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") add();
          }}
          placeholder="App to ignore (for example Discord)"
          className="min-w-0 flex-1 rounded-lg border border-white/10 bg-white/5 px-3 py-1.5 text-xs outline-none placeholder:text-white/35 focus:border-[color:var(--accent-color)]"
        />
        <button
          type="button"
          onClick={add}
          className="rounded-lg bg-[color:var(--accent-color)]/20 px-3 text-xs font-medium hover:bg-[color:var(--accent-color)]/30"
        >
          Add
        </button>
      </div>
      {apps.length > 0 && (
        <div className="flex flex-wrap gap-1.5">
          {apps.map((app) => (
            <span key={app} className="flex items-center gap-1 rounded-full bg-white/8 py-0.5 pl-2.5 pr-1 text-[11px]">
              {app}
              <button
                type="button"
                aria-label={`Stop ignoring ${app}`}
                onClick={() => onChange(apps.filter((item) => item !== app))}
                className="rounded-full p-0.5 text-white/50 hover:bg-white/10 hover:text-white"
              >
                <X size={11} />
              </button>
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
