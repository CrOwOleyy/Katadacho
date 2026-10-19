import { X } from "lucide-react";
import type { NotificationToast } from "../../lib/types/notifications";

interface NotificationBannerProps {
  toast: NotificationToast;
  /** Nombre d'autres notifications en attente derrière celle-ci. */
  pending: number;
  showBody: boolean;
  onDismiss: () => void;
}

/** Teinte stable dérivée du nom de l'application, pour reconnaître la source d'un coup d'œil. */
function hueFor(name: string): number {
  let hash = 0;
  for (const character of name) {
    hash = (hash * 31 + character.charCodeAt(0)) % 360;
  }
  return hash;
}

export function NotificationBanner({ toast, pending, showBody, onDismiss }: NotificationBannerProps) {
  const initial = toast.appName.trim().charAt(0).toUpperCase() || "?";
  const hue = hueFor(toast.appName);

  return (
    <div
      role="alert"
      onClick={(event) => {
        // Le clic ferme la notification et ne doit pas remonter jusqu'à la pilule, qui ouvrirait le menu.
        event.stopPropagation();
        onDismiss();
      }}
      className="group flex h-full w-full cursor-pointer items-center gap-3 px-4"
    >
      <div
        className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl text-sm font-semibold"
        style={{ backgroundColor: `hsl(${hue} 55% 32%)`, color: `hsl(${hue} 80% 88%)` }}
        aria-hidden="true"
      >
        {initial}
      </div>

      <div className="min-w-0 flex-1">
        <p className="truncate text-[10px] font-medium uppercase tracking-wider text-white/45">{toast.appName}</p>
        <p className="truncate text-sm font-medium leading-tight">{toast.title}</p>
        {showBody && toast.body && <p className="truncate text-xs text-white/60">{toast.body.split("\n")[0]}</p>}
      </div>

      {pending > 0 && (
        <span className="shrink-0 rounded-full bg-white/12 px-2 py-0.5 font-mono text-[10px] text-white/70">
          +{pending}
        </span>
      )}
      <X size={14} className="shrink-0 text-white/30 transition-colors group-hover:text-white/80" />
    </div>
  );
}
