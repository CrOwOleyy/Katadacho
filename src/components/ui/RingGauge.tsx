interface RingGaugeProps {
  value: number;
  label: string;
  sublabel?: string;
  size?: number;
  stroke?: number;
}

/** Anneau de progression SVG. `value` est un pourcentage entre 0 et 100. */
export function RingGauge({ value, label, sublabel, size = 76, stroke = 7 }: RingGaugeProps) {
  const clamped = Math.min(100, Math.max(0, value));
  const radius = (size - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  const offset = circumference * (1 - clamped / 100);

  return (
    <div className="flex flex-col items-center gap-1">
      <div className="relative" style={{ width: size, height: size }}>
        <svg width={size} height={size} className="-rotate-90">
          <circle cx={size / 2} cy={size / 2} r={radius} fill="none" stroke="rgba(255,255,255,0.08)" strokeWidth={stroke} />
          <circle
            cx={size / 2}
            cy={size / 2}
            r={radius}
            fill="none"
            stroke="var(--accent-color)"
            strokeWidth={stroke}
            strokeLinecap="round"
            strokeDasharray={circumference}
            strokeDashoffset={offset}
            className="transition-[stroke-dashoffset] duration-500 ease-out"
          />
        </svg>
        <span className="absolute inset-0 flex items-center justify-center font-mono text-sm font-semibold">
          {Math.round(clamped)}%
        </span>
      </div>
      <span className="text-xs font-medium text-white/80">{label}</span>
      {sublabel && <span className="truncate font-mono text-[10px] text-white/45">{sublabel}</span>}
    </div>
  );
}
