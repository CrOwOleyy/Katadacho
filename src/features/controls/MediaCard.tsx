import { Music, Pause, Play, SkipBack, SkipForward } from "lucide-react";
import { useEffect, useRef, useState, type MouseEvent } from "react";
import { BentoCard } from "../../components/ui/BentoCard";
import { useMedia } from "../../hooks/useMedia";
import { ipc } from "../../lib/ipc";
import type { MediaPosition } from "../../lib/types/media";

const POSITION_POLL_MS = 1000;
/** Fréquence de l'interpolation locale entre deux lectures de la position. */
const TICK_MS = 250;

function formatTime(totalSeconds: number): string {
  const safe = Math.max(0, Math.floor(totalSeconds));
  const minutes = Math.floor(safe / 60);
  const seconds = safe % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

/**
 * Position de lecture : lue côté Rust chaque seconde, puis interpolée localement pour que la barre
 * avance de façon fluide entre deux lectures.
 */
function usePlaybackPosition(active: boolean) {
  const [position, setPosition] = useState<MediaPosition | null>(null);
  const [now, setNow] = useState(() => Date.now());
  const fetchedAt = useRef(Date.now());

  useEffect(() => {
    if (!active) {
      setPosition(null);
      return;
    }
    let alive = true;

    const poll = () => {
      ipc
        .mediaPosition()
        .then((value) => {
          if (!alive) return;
          fetchedAt.current = Date.now();
          setPosition(value);
        })
        .catch(() => {
          if (alive) setPosition(null);
        });
    };

    poll();
    const pollId = window.setInterval(poll, POSITION_POLL_MS);
    const tickId = window.setInterval(() => setNow(Date.now()), TICK_MS);
    return () => {
      alive = false;
      window.clearInterval(pollId);
      window.clearInterval(tickId);
    };
  }, [active]);

  if (!position) return { position: null, seconds: 0 };

  const elapsed = position.playing ? (now - fetchedAt.current) / 1000 : 0;
  const raw = position.positionSeconds + elapsed;
  const seconds = position.durationSeconds ? Math.min(raw, position.durationSeconds) : raw;
  return { position, seconds };
}

export function MediaCard() {
  const media = useMedia();
  const { position, seconds } = usePlaybackPosition(media !== null);
  const [error, setError] = useState<string | null>(null);

  const run = async (action: () => Promise<void>) => {
    try {
      await action();
      setError(null);
    } catch (reason) {
      setError(String(reason));
    }
  };

  const seek = (event: MouseEvent<HTMLDivElement>) => {
    const duration = position?.durationSeconds;
    if (!duration) return;
    const bounds = event.currentTarget.getBoundingClientRect();
    const fraction = Math.min(1, Math.max(0, (event.clientX - bounds.left) / bounds.width));
    void run(() => ipc.mediaSeek(fraction * duration));
  };

  if (!media) {
    return (
      <BentoCard title="Média" icon={<Music size={14} />} className="col-span-6">
        <p className="text-sm text-white/45">Aucun média en cours de lecture</p>
      </BentoCard>
    );
  }

  const duration = position?.durationSeconds ?? null;
  const progress = duration ? (seconds / duration) * 100 : 0;
  const playing = position?.playing ?? media.playing;

  return (
    <BentoCard title="Média" icon={<Music size={14} />} className="col-span-6">
      <div className="flex items-center gap-4">
        <div className="flex h-16 w-16 shrink-0 items-center justify-center overflow-hidden rounded-xl bg-white/10">
          {media.coverBase64 ? (
            <img
              src={`data:${media.coverMime ?? "image/jpeg"};base64,${media.coverBase64}`}
              alt=""
              className="h-full w-full object-cover"
            />
          ) : (
            <Music size={24} className="text-white/40" />
          )}
        </div>

        <div className="flex min-w-0 flex-1 flex-col gap-2">
          <div className="min-w-0">
            <p className="truncate text-sm font-medium">{media.title || "Titre inconnu"}</p>
            <p className="truncate text-xs text-white/50">{media.artist || "Artiste inconnu"}</p>
          </div>

          <div className="flex items-center gap-2">
            <span className="w-9 text-right font-mono text-[10px] text-white/50">{formatTime(seconds)}</span>
            <div
              role="slider"
              aria-label="Position dans le morceau"
              aria-valuemin={0}
              aria-valuemax={duration ?? 0}
              aria-valuenow={Math.floor(seconds)}
              onClick={seek}
              className={`relative h-1.5 flex-1 overflow-hidden rounded-full bg-white/10 ${duration ? "cursor-pointer" : ""}`}
            >
              <div
                className="h-full rounded-full bg-[color:var(--accent-color)] transition-[width] duration-200 ease-linear"
                style={{ width: `${progress}%` }}
              />
            </div>
            <span className="w-9 font-mono text-[10px] text-white/50">{duration ? formatTime(duration) : "--:--"}</span>
          </div>
        </div>

        <div className="flex shrink-0 items-center gap-1">
          <button
            type="button"
            aria-label="Piste précédente"
            onClick={() => void run(ipc.mediaPrevious)}
            className="rounded-full p-2 text-white/70 transition-colors hover:bg-white/10 hover:text-white"
          >
            <SkipBack size={16} />
          </button>
          <button
            type="button"
            aria-label={playing ? "Pause" : "Lecture"}
            onClick={() => void run(ipc.mediaTogglePlayPause)}
            className="rounded-full bg-white p-2.5 text-black transition-transform hover:scale-105"
          >
            {playing ? <Pause size={16} className="fill-current" /> : <Play size={16} className="fill-current" />}
          </button>
          <button
            type="button"
            aria-label="Piste suivante"
            onClick={() => void run(ipc.mediaNext)}
            className="rounded-full p-2 text-white/70 transition-colors hover:bg-white/10 hover:text-white"
          >
            <SkipForward size={16} />
          </button>
        </div>
      </div>
      {error && <p className="truncate text-[10px] text-red-400">{error}</p>}
    </BentoCard>
  );
}
