import { AppWindow, File, Folder } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { ipc } from "../../lib/ipc";
import type { LauncherKind, LauncherResult } from "../../lib/types/launcher";

/** Cache mémoire des icônes déjà demandées, pour ne pas les redemander à chaque frappe. */
const iconCache = new Map<string, string | null>();

const KIND_LABEL: Record<LauncherKind, string> = {
  app: "Application",
  uwp: "Microsoft Store",
  file: "Fichier",
  folder: "Dossier",
};

function FallbackIcon({ kind }: { kind: LauncherKind }) {
  const Icon = kind === "folder" ? Folder : kind === "file" ? File : AppWindow;
  return <Icon size={20} className="text-white/50" />;
}

function useIcon(path: string): string | null {
  const [icon, setIcon] = useState<string | null>(iconCache.get(path) ?? null);

  useEffect(() => {
    if (iconCache.has(path)) {
      setIcon(iconCache.get(path) ?? null);
      return;
    }
    let active = true;
    ipc
      .getLauncherIcon(path)
      .then((base64) => {
        iconCache.set(path, base64);
        if (active) setIcon(base64);
      })
      .catch(() => {
        // Pas d'icône disponible : on mémorise l'échec et on garde l'icône générique.
        iconCache.set(path, null);
      });
    return () => {
      active = false;
    };
  }, [path]);

  return icon;
}

interface ResultRowProps {
  result: LauncherResult;
  selected: boolean;
  onActivate: () => void;
  onHover: () => void;
}

export function ResultRow({ result, selected, onActivate, onHover }: ResultRowProps) {
  const icon = useIcon(result.path);
  const ref = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (selected) ref.current?.scrollIntoView({ block: "nearest" });
  }, [selected]);

  return (
    <button
      ref={ref}
      type="button"
      onClick={onActivate}
      onMouseEnter={onHover}
      className={`flex w-full items-center gap-3 rounded-xl px-3 py-2 text-left transition-colors ${
        selected ? "bg-[color:var(--accent-color)]/20" : "hover:bg-white/5"
      }`}
    >
      <div className="flex h-8 w-8 shrink-0 items-center justify-center">
        {icon ? (
          <img src={`data:image/png;base64,${icon}`} alt="" className="h-7 w-7 object-contain" />
        ) : (
          <FallbackIcon kind={result.kind} />
        )}
      </div>
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm">{result.name}</p>
        <p className="truncate text-[10px] text-white/40">
          {KIND_LABEL[result.kind]}
          {result.kind === "file" || result.kind === "folder" ? ` · ${result.path}` : ""}
        </p>
      </div>
    </button>
  );
}
