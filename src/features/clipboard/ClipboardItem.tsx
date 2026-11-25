import { Lock, Pin, Trash2 } from "lucide-react";
import type { ClipboardItem as Item } from "../../lib/types/clipboard";

interface ClipboardItemProps {
  item: Item;
  copied: boolean;
  onCopy: () => void;
  onPaste: () => void;
  onTogglePin: () => void;
  onDelete: () => void;
  /** Minutes avant l'effacement automatique d'un secret non épinglé. */
  secretTtlMinutes: number;
}

function relativeTime(timestamp: number): string {
  const seconds = Math.max(0, Math.round((Date.now() - timestamp) / 1000));
  if (seconds < 60) return "just now";
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes} min ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours} h ago`;
  return `${Math.round(hours / 24)} d ago`;
}

function Preview({ item }: { item: Item }) {
  // Un secret ne s'affiche jamais en clair dans la liste, il reste copiable et collable.
  if (item.isSensitive) {
    return (
      <div className="flex items-center gap-2">
        <Lock size={14} className="shrink-0 text-amber-300/80" />
        <span className="font-mono text-sm tracking-widest text-white/70">••••••••••••</span>
      </div>
    );
  }

  if (item.contentType === "image" && item.thumbnailBase64) {
    return (
      <img
        src={`data:image/png;base64,${item.thumbnailBase64}`}
        alt="Copied image"
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

function subtitle(item: Item, secretTtlMinutes: number): string {
  const origin = `${item.sourceAppName ?? "Unknown"} · ${relativeTime(item.createdAt)}`;
  if (!item.isSensitive) return origin;
  if (item.isPinned) return `Pinned secret · ${origin}`;
  const minutesLeft = Math.max(0, Math.ceil((item.createdAt + secretTtlMinutes * 60_000 - Date.now()) / 60_000));
  return `Secret, erased in ${minutesLeft} min · ${origin}`;
}

export function ClipboardItem({ item, copied, onCopy, onPaste, onTogglePin, onDelete, secretTtlMinutes }: ClipboardItemProps) {
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
          {copied ? "Copied" : subtitle(item, secretTtlMinutes)}
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
          Paste
        </button>
        <button
          type="button"
          aria-label={item.isPinned ? "Unpin" : "Pin"}
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
          aria-label="Delete"
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
