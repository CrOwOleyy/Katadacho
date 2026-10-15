import { GHOST_COLORS as C } from "./palette";

/** Silhouette : dôme arrondi et bas ondulé à trois festons. Repère : 100 de large, corps de 10 à 96. */
export const BODY_PATH =
  "M20 88 V46 C20 24 33 10 50 10 C67 10 80 24 80 46 V88 Q73 96 66.5 88 Q60 80 53.5 88 Q47 96 40.5 88 Q34 80 27.5 88 Q23 93 20 88 Z";

/** Ombre portée sur le côté droit, découpée par la silhouette pour ne jamais déborder. */
const SHADE_PATH = "M63 10 C80 22 82 40 80 96 H60 C72 66 72 34 63 10 Z";

interface ArmProps {
  /** Point d'attache à l'épaule. */
  x: number;
  y: number;
  /** Angle en degrés : 0 le bras pend, 90 il est à l'horizontale, 150 il est levé. */
  angle: number;
  side: "left" | "right";
}

/** Un bras court, qui pivote autour de l'épaule. */
export function Arm({ x, y, angle, side }: ArmProps) {
  const rotation = side === "left" ? angle : -angle;
  return (
    <g transform={`translate(${x} ${y})`}>
      <g style={{ transform: `rotate(${rotation}deg)`, transformOrigin: "0px 0px", transition: "transform 0.35s cubic-bezier(.3,1.5,.5,1)" }}>
        <ellipse cx={0} cy={7} rx={5.2} ry={8.6} fill={C.cream} />
      </g>
    </g>
  );
}

export function Body({ armAngle, showArms }: { armAngle: number; showArms: boolean }) {
  return (
    <g>
      {showArms && (
        <>
          <Arm x={21} y={58} angle={armAngle + 14} side="left" />
          <Arm x={79} y={58} angle={armAngle + 14} side="right" />
        </>
      )}
      <defs>
        <clipPath id="ghost-body-clip">
          <path d={BODY_PATH} />
        </clipPath>
      </defs>
      <path d={BODY_PATH} fill={C.cream} />
      <path d={SHADE_PATH} fill={C.creamShade} clipPath="url(#ghost-body-clip)" opacity={0.75} />
      {/* Petit reflet sur le haut de la tête : seul détail de volume. */}
      <ellipse cx={36} cy={25} rx={4.2} ry={8} transform="rotate(18 36 25)" fill="#FFFFFF" opacity={0.5} />
    </g>
  );
}
