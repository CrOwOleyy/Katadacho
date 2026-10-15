import { GHOST_COLORS as C } from "./palette";

const CX = 50;
const CY = 70;
const HALF_WIDTH = 9;
const HALF_HEIGHT = 12;

/** Gemme en losange à quatre facettes, la signature de la mascotte. */
export function Gem() {
  const top = `${CX},${CY - HALF_HEIGHT}`;
  const right = `${CX + HALF_WIDTH},${CY}`;
  const bottom = `${CX},${CY + HALF_HEIGHT}`;
  const left = `${CX - HALF_WIDTH},${CY}`;
  const centre = `${CX},${CY}`;

  return (
    <g strokeLinejoin="round">
      <polygon points={`${top} ${left} ${centre}`} fill={C.gem.blue} />
      <polygon points={`${top} ${right} ${centre}`} fill={C.gem.red} />
      <polygon points={`${left} ${bottom} ${centre}`} fill={C.gem.green} />
      <polygon points={`${right} ${bottom} ${centre}`} fill={C.gem.yellow} />
      <polygon
        points={`${top} ${right} ${bottom} ${left}`}
        fill="none"
        stroke="#FFFFFF"
        strokeOpacity={0.55}
        strokeWidth={0.8}
      />
      <path d={`M${CX - 3} ${CY - 6} L${CX - 0.5} ${CY - 9}`} stroke="#FFFFFF" strokeOpacity={0.7} strokeWidth={1} strokeLinecap="round" />
    </g>
  );
}
