import { Pin, Trash2 } from "lucide-react";
import type { ClipboardItem as Item } from "../../lib/types/clipboard";

interface ClipboardItemProps {
  item: Item;
  copied: boolean;
  onCopy: () => void;
  onPaste: () => void;
  onTogglePin: () => void;
  onDelete: () => void;
}

function relativeTime(timestamp: number): string {
  const seconds = Math.max(0, Math.round((Date.now() - timestamp) / 1000));
  if (seconds < 60) return "à l'instant";
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `il y a ${minutes} min`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `il y a ${hours} h`;
  return `il y a ${Math.round(hours / 24)} j`;
}

function Preview({ item }: { item: Item }) {
  if (item.contentType === "image" && item.thumbnailBase64) {
    return (
      <img
        src={`data:image/png;base64,${item.thumbnailBase64}`}
        alt="Image copiée"
        className="h-12 w-20 shrink-0 rounded-md bg-white/5 object-cover"
      />
    );
  }

  if (item.contentType === "color" && item.textContent) {
    const color = item.textContent.trim();
    return (
      <div className="flex items-center gap-2">
        <span className="h-8 w-8 shrink-0 rounded-md border border-white/15" style={{ backgroundColor: color }} />
        <span className="font-mono text-sm">{color.toUpperCase()}</span>
      </div>
    );
  }

  const isCode = item.contentType === "code";
  return (
    <p
      className={`line-clamp-2 min-w-0 break-all text-xs ${isCode ? "font-mono text-white/80" : "text-white/90"} ${
        item.contentType === "url" ? "text-[color:var(--accent-color)]" : ""
      }`}
    >
      {item.textContent}
    </p>
  );
}

export function ClipboardItem({ item, copied, onCopy, onPaste, onTogglePin, onDelete }: ClipboardItemProps) {
  return (
    <div
      role="button"
      tabIndex={0}
      onClick={onCopy}
      onKeyDown={(event) => {
        if (event.key === "Enter") onCopy();
      }}
      className="group flex cursor-pointer items-center gap-3 rounded-xl border border-transparent bg-white/[0.03] px-3 py-2 transition-colors hover:border-white/10 hover:bg-white/[0.06]"
    >
      <div className="min-w-0 flex-1">
        <Preview item={item} />
        <p className="mt-1 text-[10px] text-white/40">
          {copied ? "Copié" : `${item.sourceAppName ?? "Inconnu"} · ${relativeTime(item.createdAt)}`}
        </p>
      </div>

      <div className="flex shrink-0 items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100">
        <button
          type="button"
          onClick={(event) => {
            event.stopPropagation();
            onPaste();
          }}
          className="rounded-md bg-[color:var(--accent-color)]/20 px-2 py-0.5 text-[10px] font-medium hover:bg-[color:var(--accent-color)]/35"
        >
          Coller
        </button>
        <button
          type="button"
          aria-label={item.isPinned ? "Désépingler" : "Épingler"}
          onClick={(event) => {
            event.stopPropagation();
            onTogglePin();
          }}
          className="rounded-md p-1 text-white/60 hover:bg-white/10 hover:text-white"
        >
          <Pin size={13} className={item.isPinned ? "fill-current text-[color:var(--accent-color)]" : ""} />
        </button>
        <button
          type="button"
          aria-label="Supprimer"
          onClick={(event) => {
            event.stopPropagation();
            onDelete();
          }}
          className="rounded-md p-1 text-white/60 hover:bg-red-500/20 hover:text-red-300"
        >
          <Trash2 size={13} />
        </button>
      </div>

      {item.isPinned && (
        <Pin size={12} className="shrink-0 fill-current text-[color:var(--accent-color)] group-hover:hidden" />
      )}
    </div>
  );
}
