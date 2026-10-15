interface SpeedGaugeProps {
  mbps: number;
  maxMbps?: number;
  label: string;
}

/** Jauge en demi-cercle. L'arc se remplit proportionnellement au débit par rapport à `maxMbps`. */
export function SpeedGauge({ mbps, maxMbps = 1000, label }: SpeedGaugeProps) {
  const size = 140;
  const stroke = 10;
  const radius = (size - stroke) / 2;
  const arcLength = Math.PI * radius;
  const ratio = Math.min(1, Math.max(0, mbps / maxMbps));
  const offset = arcLength * (1 - ratio);

  return (
    <div className="flex flex-col items-center">
      <svg width={size} height={size / 2 + stroke} viewBox={`0 0 ${size} ${size / 2 + stroke}`}>
        <path
          d={`M ${stroke / 2} ${size / 2} A ${radius} ${radius} 0 0 1 ${size - stroke / 2} ${size / 2}`}
          fill="none"
          stroke="rgba(255,255,255,0.08)"
          strokeWidth={stroke}
          strokeLinecap="round"
        />
        <path
          d={`M ${stroke / 2} ${size / 2} A ${radius} ${radius} 0 0 1 ${size - stroke / 2} ${size / 2}`}
          fill="none"
          stroke="var(--accent-color)"
          strokeWidth={stroke}
          strokeLinecap="round"
          strokeDasharray={arcLength}
          strokeDashoffset={offset}
          className="transition-[stroke-dashoffset] duration-200 ease-out"
        />
      </svg>
      <span className="-mt-7 font-mono text-xl font-semibold">{mbps.toFixed(1)}</span>
      <span className="text-[10px] uppercase tracking-wider text-white/50">{label}</span>
    </div>
  );
}
