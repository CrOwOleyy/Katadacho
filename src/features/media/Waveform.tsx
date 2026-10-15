import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";
import { ipc } from "../../lib/ipc";

interface WaveformProps {
  /** Vrai tant qu'un son est en lecture : démarre le flux de niveaux côté Rust. */
  active: boolean;
  width?: number;
  height?: number;
}

const COLOR = "#ef4444";
const BAR_WIDTH = 2;
const BAR_GAP = 1.5;
const MIN_BAR_HEIGHT = 2;
/** Plancher du normaliseur : évite d'amplifier le bruit quand rien ne joue. */
const MIN_REFERENCE_PEAK = 0.05;
/** Décroissance de la référence, pour s'adapter si le volume baisse. */
const REFERENCE_DECAY = 0.995;

/**
 * Onde sonore rouge qui défile, dessinée à partir du niveau réel de la sortie audio.
 * Les niveaux sont normalisés par rapport au pic récent, donc elle reste lisible quel que soit
 * le volume système.
 */
export function Waveform({ active, width = 72, height = 22 }: WaveformProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const context = canvas?.getContext("2d");
    if (!canvas || !context) return;

    const ratio = window.devicePixelRatio || 1;
    canvas.width = width * ratio;
    canvas.height = height * ratio;
    context.setTransform(ratio, 0, 0, ratio, 0, 0);

    const barCount = Math.floor(width / (BAR_WIDTH + BAR_GAP));
    const levels = new Array<number>(barCount).fill(0);
    let reference = MIN_REFERENCE_PEAK;

    const draw = () => {
      context.clearRect(0, 0, width, height);
      context.fillStyle = COLOR;
      levels.forEach((level, index) => {
        const barHeight = Math.max(MIN_BAR_HEIGHT, Math.pow(level, 0.6) * height);
        context.beginPath();
        context.roundRect(index * (BAR_WIDTH + BAR_GAP), (height - barHeight) / 2, BAR_WIDTH, barHeight, 1);
        context.fill();
      });
    };

    draw();
    if (!active) return;

    let unlisten: (() => void) | undefined;
    let alive = true;

    listen<number>("audio-level", (event) => {
      reference = Math.max(reference * REFERENCE_DECAY, event.payload, MIN_REFERENCE_PEAK);
      levels.push(Math.min(1, event.payload / reference));
      levels.shift();
      draw();
    })
      .then((fn) => {
        if (alive) unlisten = fn;
        else fn();
      })
      .catch((reason: unknown) => console.error("audio-level listener failed", reason));

    ipc.startAudioLevels().catch((reason: unknown) => console.error("start_audio_levels failed", reason));

    return () => {
      alive = false;
      unlisten?.();
      ipc.stopAudioLevels().catch((reason: unknown) => console.error("stop_audio_levels failed", reason));
    };
  }, [active, width, height]);

  return <canvas ref={canvasRef} style={{ width, height }} aria-hidden="true" />;
}
