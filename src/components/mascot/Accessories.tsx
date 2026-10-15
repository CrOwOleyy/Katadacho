import { motion } from "motion/react";
import { GHOST_COLORS as C } from "./palette";
import type { GhostMood } from "./mood";

interface AccessoriesProps {
  mood: GhostMood;
  animated: boolean;
  /** Multiplicateur de vitesse des animations, 1 par défaut. */
  speed: number;
}

function Loop({ animated, speed, delay = 0, duration, children, keyframes }: {
  animated: boolean;
  speed: number;
  delay?: number;
  duration: number;
  children: React.ReactNode;
  keyframes: Record<string, number[]>;
}) {
  const rest = Object.fromEntries(Object.entries(keyframes).map(([key, values]) => [key, values[0]]));
  return (
    <motion.g
      animate={animated ? keyframes : rest}
      transition={{ duration: duration / speed, delay: delay / speed, repeat: Infinity, ease: "easeInOut" }}
    >
      {children}
    </motion.g>
  );
}

function Headphones() {
  return (
    <g fill="none" strokeLinecap="round">
      <path d="M21.5 42 C21.5 14 78.5 14 78.5 42" stroke={C.headphones} strokeWidth={3.4} />
      <rect x={15.5} y={36} width={9} height={17} rx={4.5} fill={C.headphones} />
      <rect x={75.5} y={36} width={9} height={17} rx={4.5} fill={C.headphones} />
    </g>
  );
}

function Note({ x, y }: { x: number; y: number }) {
  return (
    <g transform={`translate(${x} ${y})`} fill={C.cream}>
      <ellipse cx={0} cy={6} rx={2.6} ry={2} />
      <rect x={1.6} y={-5} width={1.2} height={11} rx={0.6} />
      <path d="M2.8 -5 Q6.5 -3.5 6 0 Q4.5 -2 2.8 -2.2 Z" />
    </g>
  );
}

function Headband() {
  return (
    <g fill="none" strokeLinecap="round">
      <path d="M21.5 30 Q50 21 78.5 30" stroke={C.band} strokeWidth={4.2} />
      <path d="M78 29.5 L89 24" stroke={C.band} strokeWidth={3} />
      <path d="M78 30.5 L88 34.5" stroke={C.band} strokeWidth={3} />
    </g>
  );
}

function SleepZ({ x, y, size }: { x: number; y: number; size: number }) {
  return (
    <path
      d={`M${x} ${y} h${size} l-${size} ${size} h${size}`}
      fill="none"
      stroke={C.cream}
      strokeWidth={1.7}
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  );
}

const CONFETTI = [
  { x: 14, y: 14, color: C.gem.blue },
  { x: 86, y: 12, color: C.gem.yellow },
  { x: 8, y: 34, color: C.gem.red },
  { x: 92, y: 32, color: C.gem.green },
  { x: 30, y: 4, color: C.gem.yellow },
  { x: 70, y: 3, color: C.gem.red },
];

export function Accessories({ mood, animated, speed }: AccessoriesProps) {
  switch (mood) {
    case "thinking":
      return (
        <g fill={C.cream}>
          {[
            { x: 76, y: 17, r: 1.7, delay: 0 },
            { x: 83, y: 11, r: 2.1, delay: 0.25 },
            { x: 91, y: 5, r: 2.6, delay: 0.5 },
          ].map((dot) => (
            <Loop key={dot.x} animated={animated} speed={speed} duration={1.4} delay={dot.delay} keyframes={{ opacity: [0.25, 1, 0.25] }}>
              <circle cx={dot.x} cy={dot.y} r={dot.r} />
            </Loop>
          ))}
        </g>
      );
    case "alert":
      return (
        <Loop animated={animated} speed={speed} duration={0.6} keyframes={{ rotate: [-6, 6, -6] }}>
          <g style={{ transformOrigin: "82px 12px" }}>
            <rect x={80.7} y={1} width={2.8} height={9} rx={1.4} fill={C.coral} />
            <circle cx={82.1} cy={14} r={1.7} fill={C.coral} />
          </g>
        </Loop>
      );
    case "music":
      return (
        <g>
          <Headphones />
          <Loop animated={animated} speed={speed} duration={2.2} keyframes={{ y: [0, -5, 0], opacity: [0.4, 1, 0.4] }}>
            <Note x={8} y={20} />
          </Loop>
          <Loop animated={animated} speed={speed} duration={2.2} delay={1.1} keyframes={{ y: [0, -5, 0], opacity: [0.4, 1, 0.4] }}>
            <Note x={88} y={14} />
          </Loop>
        </g>
      );
    case "focus":
      return <Headband />;
    case "sleepy":
      return (
        <g>
          <Loop animated={animated} speed={speed} duration={2.6} keyframes={{ y: [0, -4, 0], opacity: [0.35, 1, 0.35] }}>
            <SleepZ x={72} y={16} size={5} />
          </Loop>
          <Loop animated={animated} speed={speed} duration={2.6} delay={1.3} keyframes={{ y: [0, -4, 0], opacity: [0.35, 1, 0.35] }}>
            <SleepZ x={81} y={7} size={7} />
          </Loop>
        </g>
      );
    case "happy":
      return (
        <g>
          {CONFETTI.map((piece, index) => (
            <Loop key={index} animated={animated} speed={speed} duration={1.2} delay={index * 0.15} keyframes={{ y: [0, 4, 0], opacity: [1, 0.5, 1] }}>
              <rect x={piece.x} y={piece.y} width={3} height={3} rx={0.6} fill={piece.color} transform={`rotate(${index * 35} ${piece.x} ${piece.y})`} />
            </Loop>
          ))}
        </g>
      );
    default:
      return null;
  }
}
