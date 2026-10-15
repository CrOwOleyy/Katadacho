import { GHOST_COLORS as C } from "./palette";

const CENTER_X = 50;
const RING_HEIGHT = 3.6;

/** Anneaux du ressort, du plus large (sous le corps) au plus étroit. */
const RINGS_FULL = [
  { y: 94, radius: 14 },
  { y: 102, radius: 12.5 },
  { y: 110, radius: 11 },
  { y: 118, radius: 9 },
];
/** Version simplifiée pour les petites tailles : moins d'anneaux, traits plus épais. */
const RINGS_MINI = [
  { y: 95, radius: 13 },
  { y: 105, radius: 10.5 },
  { y: 115, radius: 8 },
];

/** Ressort en anneaux : la moitié arrière est plus sombre, la moitié avant plus claire. */
export function Spring({ mini }: { mini: boolean }) {
  const rings = mini ? RINGS_MINI : RINGS_FULL;
  const backWidth = mini ? 4 : 3.2;
  const frontWidth = mini ? 4.6 : 3.8;

  return (
    <g fill="none" strokeLinecap="round">
      {rings.map(({ y, radius }) => {
        const left = CENTER_X - radius;
        const right = CENTER_X + radius;
        return (
          <g key={y}>
            <path d={`M${left} ${y} A${radius} ${RING_HEIGHT} 0 0 1 ${right} ${y}`} stroke={C.springBack} strokeWidth={backWidth} />
            <path d={`M${left} ${y} A${radius} ${RING_HEIGHT} 0 0 0 ${right} ${y}`} stroke={C.springFront} strokeWidth={frontWidth} />
          </g>
        );
      })}
    </g>
  );
}
