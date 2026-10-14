import { useEffect, useRef, useState } from "react";

interface SliderProps {
  value: number;
  min?: number;
  max?: number;
  label: string;
  onCommit: (value: number) => void;
}

/**
 * Curseur fluide : l'affichage suit le doigt immédiatement, mais la commande
 * système n'est envoyée qu'après une courte pause, pour ne pas saturer le matériel.
 */
export function Slider({ value, min = 0, max = 100, label, onCommit }: SliderProps) {
  const [local, setLocal] = useState(value);
  const [dragging, setDragging] = useState(false);
  const timer = useRef<number | undefined>(undefined);

  useEffect(() => {
    if (!dragging) setLocal(value);
  }, [value, dragging]);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  const handleChange = (next: number) => {
    setLocal(next);
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => onCommit(next), 80);
  };

  const percent = ((local - min) / (max - min)) * 100;

  return (
    <input
      type="range"
      aria-label={label}
      min={min}
      max={max}
      value={local}
      onChange={(event) => handleChange(Number(event.target.value))}
      onPointerDown={() => setDragging(true)}
      onPointerUp={() => setDragging(false)}
      onPointerCancel={() => setDragging(false)}
      className="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-white/10 accent-[color:var(--accent-color)]"
      style={{
        background: `linear-gradient(to right, var(--accent-color) ${percent}%, rgba(255,255,255,0.1) ${percent}%)`,
      }}
    />
  );
}
