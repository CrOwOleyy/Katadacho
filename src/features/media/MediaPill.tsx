import { Music } from "lucide-react";
import { Soul } from "../../components/soul/Soul";
import type { MediaInfo } from "../../lib/types/media";
import { Waveform } from "./Waveform";

interface MediaPillProps {
  media: MediaInfo | null;
  time: string;
}

/** Contenu de la pilule repliée : Soul, pochette et onde sonore du média en cours, puis l'heure. */
export function MediaPill({ media, time }: MediaPillProps) {
  const label = media ? [media.title, media.artist].filter(Boolean).join(" · ") : undefined;

  return (
    <div className="flex h-full w-full items-center justify-between gap-2 px-3" title={label}>
      <Soul />

      {media && (
        <>
          <div className="flex h-[26px] w-[26px] shrink-0 items-center justify-center overflow-hidden rounded-md bg-white/10">
            {media.coverBase64 ? (
              <img
                src={`data:${media.coverMime ?? "image/jpeg"};base64,${media.coverBase64}`}
                alt=""
                className="h-full w-full object-cover"
              />
            ) : (
              <Music size={14} className="text-white/60" />
            )}
          </div>
          <Waveform active={media.playing} />
        </>
      )}

      <span className="font-mono text-xs text-white/70">{time}</span>
    </div>
  );
}
