import { motion } from "motion/react";
import { GHOST_COLORS as C } from "./palette";
import type { GhostMood } from "./mood";

const EYE_Y = 42;
const LEFT_EYE_X = 40;
const RIGHT_EYE_X = 60;
const STROKE = 1.9;

interface FaceProps {
  mood: GhostMood;
  /** Vrai pendant un clignement : les yeux ronds s'aplatissent. */
  blinking: boolean;
  /** Décalage du regard, en unités du dessin (environ -3 à 3). */
  look: { x: number; y: number };
  mini: boolean;
}

function Stroke({ d }: { d: string }) {
  return <path d={d} fill="none" stroke={C.ink} strokeWidth={STROKE} strokeLinecap="round" strokeLinejoin="round" />;
}

function RoundEyes({ blinking, ry = 3.6 }: { blinking: boolean; ry?: number }) {
  return (
    <>
      <motion.ellipse cx={LEFT_EYE_X} cy={EYE_Y} rx={2.8} fill={C.ink} animate={{ ry: blinking ? 0.5 : ry }} transition={{ duration: 0.07 }} />
      <motion.ellipse cx={RIGHT_EYE_X} cy={EYE_Y} rx={2.8} fill={C.ink} animate={{ ry: blinking ? 0.5 : ry }} transition={{ duration: 0.07 }} />
    </>
  );
}

function Eyes({ mood, blinking }: { mood: GhostMood; blinking: boolean }) {
  switch (mood) {
    case "alert":
      return (
        <>
          {[LEFT_EYE_X, RIGHT_EYE_X].map((x) => (
            <g key={x}>
              <circle cx={x} cy={EYE_Y} r={4.9} fill="#FFFFFF" stroke={C.ink} strokeWidth={1.5} />
              <circle cx={x} cy={EYE_Y} r={1.9} fill={C.ink} />
            </g>
          ))}
        </>
      );
    case "sleepy":
      return (
        <>
          <Stroke d={`M${LEFT_EYE_X - 4.5} ${EYE_Y + 1} Q${LEFT_EYE_X} ${EYE_Y + 5} ${LEFT_EYE_X + 4.5} ${EYE_Y + 1}`} />
          <Stroke d={`M${RIGHT_EYE_X - 4.5} ${EYE_Y + 1} Q${RIGHT_EYE_X} ${EYE_Y + 5} ${RIGHT_EYE_X + 4.5} ${EYE_Y + 1}`} />
        </>
      );
    case "happy":
      return (
        <>
          <Stroke d={`M${LEFT_EYE_X - 4.5} ${EYE_Y + 2} Q${LEFT_EYE_X} ${EYE_Y - 4} ${LEFT_EYE_X + 4.5} ${EYE_Y + 2}`} />
          <Stroke d={`M${RIGHT_EYE_X - 4.5} ${EYE_Y + 2} Q${RIGHT_EYE_X} ${EYE_Y - 4} ${RIGHT_EYE_X + 4.5} ${EYE_Y + 2}`} />
        </>
      );
    case "sad":
      return <RoundEyes blinking={blinking} ry={3} />;
    default:
      return <RoundEyes blinking={blinking} />;
  }
}

function Brows({ mood }: { mood: GhostMood }) {
  if (mood === "focus") {
    return (
      <>
        <Stroke d="M34.5 35.5 L45 39" />
        <Stroke d="M65.5 35.5 L55 39" />
      </>
    );
  }
  if (mood === "sad") {
    return (
      <>
        <Stroke d="M34.5 38.5 L45 35" />
        <Stroke d="M65.5 38.5 L55 35" />
      </>
    );
  }
  return null;
}

function Mouth({ mood }: { mood: GhostMood }) {
  switch (mood) {
    case "alert":
      return <ellipse cx={50} cy={54} rx={2.4} ry={3.2} fill={C.ink} />;
    case "thinking":
      return <ellipse cx={53} cy={53.5} rx={1.7} ry={2.2} fill={C.ink} />;
    case "focus":
      return <Stroke d="M45 53.5 H55" />;
    case "sleepy":
      return <Stroke d="M46.5 53 Q50 55.5 53.5 53" />;
    case "happy":
      return <path d="M42 50.5 Q50 63 58 50.5 Z" fill={C.ink} strokeLinejoin="round" stroke={C.ink} strokeWidth={1.2} />;
    case "sad":
      return <Stroke d="M44 56.5 Q50 51 56 56.5" />;
    case "watching":
      // Bouche ouverte à moitié : le fantôme grignote son pop-corn.
      return <ellipse cx={50} cy={53} rx={2.6} ry={1.8} fill={C.ink} />;
    default:
      return <Stroke d="M43 51.5 Q50 58 57 51.5" />;
  }
}

/** Petit cœur sur la joue droite. */
function Heart() {
  return (
    <path
      d="M68.5 53.4 C65 50.8 64 49.2 64 47.8 C64 46.5 65 45.7 66.1 45.7 C67.1 45.7 68 46.2 68.5 47 C69 46.2 69.9 45.7 70.9 45.7 C72 45.7 73 46.5 73 47.8 C73 49.2 72 50.8 68.5 53.4 Z"
      fill={C.coral}
    />
  );
}

export function Face({ mood, blinking, look, mini }: FaceProps) {
  return (
    <g>
      {/* Le regard glisse avec un ressort au lieu de sauter d'une position à l'autre. */}
      <motion.g animate={{ x: look.x, y: look.y }} transition={{ type: "spring", stiffness: 260, damping: 18 }}>
        <Eyes mood={mood} blinking={blinking} />
        <Brows mood={mood} />
      </motion.g>
      <Mouth mood={mood} />
      {!mini && mood !== "alert" && mood !== "sad" && <Heart />}
      {mood === "sad" && <path d="M61 47 Q64 52 61 55 Q58 52 61 47 Z" fill={C.tear} />}
    </g>
  );
}
