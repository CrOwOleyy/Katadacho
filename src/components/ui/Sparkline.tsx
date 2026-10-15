interface SparklineProps {
  values: number[];
  max?: number;
  height?: number;
}

/** Mini graphique linéaire. `max` fixe l'échelle verticale, 100 par défaut pour des %. */
export function Sparkline({ values, max = 100, height = 28 }: SparklineProps) {
  const width = 120;
  if (values.length < 2) {
    return <div style={{ height }} className="w-full" />;
  }

  const step = width / (values.length - 1);
  const points = values
    .map((value, index) => {
      const y = height - (Math.min(value, max) / max) * height;
      return `${(index * step).toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");

  return (
    <svg viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="none" className="h-7 w-full overflow-visible">
      <polyline
        points={points}
        fill="none"
        stroke="var(--accent-color)"
        strokeWidth={1.5}
        strokeLinejoin="round"
        vectorEffect="non-scaling-stroke"
      />
    </svg>
  );
}
