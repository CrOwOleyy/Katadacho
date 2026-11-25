import { SlidersHorizontal, Volume2, VolumeX } from "lucide-react";
import { BentoCard } from "../../components/ui/BentoCard";
import { Slider } from "../../components/ui/Slider";
import { useAppIcon } from "../../hooks/useAppIcon";
import type { AppVolume } from "../../lib/types/controls";

interface MixerCardProps {
  apps: AppVolume[];
  onVolume: (id: string, level: number) => void;
  onToggleMute: (id: string) => void;
}

/** Icône de l'application, ou son initiale tant qu'elle charge ou si elle n'en a pas. */
function AppIcon({ app }: { app: AppVolume }) {
  const icon = useAppIcon(app.iconPath);
  if (icon) {
    return <img src={`data:image/png;base64,${icon}`} alt="" className="h-6 w-6 shrink-0 object-contain" />;
  }
  return (
    <span
      aria-hidden
      className="flex h-6 w-6 shrink-0 items-center justify-center rounded-lg bg-white/10 text-[11px] font-semibold text-white/70"
    >
      {app.name.charAt(0).toUpperCase()}
    </span>
  );
}

/** Volume de chaque application qui joue du son, réglable séparément du volume principal. */
export function MixerCard({ apps, onVolume, onToggleMute }: MixerCardProps) {
  return (
    <BentoCard title="Mixeur" icon={<SlidersHorizontal size={14} />} className="col-span-6">
      {apps.length === 0 ? (
        <p className="text-sm text-white/45">Aucune application ne joue de son</p>
      ) : (
        <ul className="flex flex-col gap-2.5">
          {apps.map((app) => (
            <li key={app.id} className="flex items-center gap-3">
              <AppIcon app={app} />
              <span className="w-36 shrink-0 truncate text-sm text-white/80" title={app.name}>
                {app.name}
              </span>
              <button
                type="button"
                onClick={() => onToggleMute(app.id)}
                aria-label={app.muted ? `Réactiver le son de ${app.name}` : `Couper le son de ${app.name}`}
                className="rounded-lg p-1 text-white/70 transition-colors hover:bg-white/10 hover:text-white"
              >
                {app.muted ? <VolumeX size={16} /> : <Volume2 size={16} />}
              </button>
              <Slider value={app.volume} label={`Volume de ${app.name}`} onCommit={(level) => onVolume(app.id, level)} />
              <span className="w-9 text-right font-mono text-xs text-white/60">{app.volume}</span>
            </li>
          ))}
        </ul>
      )}
    </BentoCard>
  );
}
