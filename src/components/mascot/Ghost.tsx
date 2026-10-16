import { motion } from "motion/react";
import { useEffect, useState } from "react";
import { Accessories } from "./Accessories";
import { Body } from "./Body";
import { Face } from "./Face";
import { Gem } from "./Gem";
import type { GhostMood } from "./mood";
import { Spring } from "./Spring";

const VIEW_WIDTH = 100;
const VIEW_HEIGHT = 128;
/** Sous cette hauteur en pixels, on passe à la version simplifiée (moins de détails). */
const MINI_THRESHOLD = 40;
const BLINK_MIN_MS = 2500;
const BLINK_MAX_MS = 6000;
const BLINK_DURATION_MS = 140;

/** Humeurs où les yeux sont des disques qui peuvent cligner. */
const BLINKING_MOODS: GhostMood[] = ["idle", "thinking", "music", "focus", "sad"];

function useBlink(enabled: boolean): boolean {
  const [blinking, setBlinking] = useState(false);

  useEffect(() => {
    if (!enabled) {
      setBlinking(false);
      return;
    }
    let openTimer = 0;
    let closeTimer = 0;

    const schedule = () => {
      openTimer = window.setTimeout(() => {
        setBlinking(true);
        closeTimer = window.setTimeout(() => {
          setBlinking(false);
          schedule();
        }, BLINK_DURATION_MS);
      }, BLINK_MIN_MS + Math.random() * (BLINK_MAX_MS - BLINK_MIN_MS));
    };

    schedule();
    return () => {
      window.clearTimeout(openTimer);
      window.clearTimeout(closeTimer);
    };
  }, [enabled]);

  return blinking;
}

export interface GhostProps {
  /** Hauteur en pixels. La largeur suit les proportions du dessin. */
  size?: number;
  mood?: GhostMood;
  /** Désactive le flottement, le clignement et les accessoires animés. */
  animated?: boolean;
  speed?: number;
  look?: { x: number; y: number };
  className?: string;
}

const NO_LOOK = { x: 0, y: 0 };

export function Ghost({ size = 64, mood = "idle", animated = true, speed = 1, look = NO_LOOK, className }: GhostProps) {
  const mini = size < MINI_THRESHOLD;
  const blinking = useBlink(animated && BLINKING_MOODS.includes(mood));

  const armAngle = mood === "happy" ? 125 : mood === "alert" ? 35 : 0;
  const excited = mood === "happy" || mood === "alert";
  const sleepy = mood === "sleepy";

  const floatDuration = (excited ? 0.8 : sleepy ? 4.2 : 2.6) / speed;
  const floatHeight = excited ? 6 : sleepy ? 1.2 : 2;

  return (
    <svg
      viewBox={`0 0 ${VIEW_WIDTH} ${VIEW_HEIGHT}`}
      width={(size * VIEW_WIDTH) / VIEW_HEIGHT}
      height={size}
      className={className}
      role="img"
      aria-label="Mascotte fantôme"
    >
      <motion.g
        animate={animated ? { y: [0, -floatHeight, 0] } : { y: 0 }}
        transition={{ duration: floatDuration, repeat: Infinity, ease: "easeInOut" }}
      >
        <motion.g
          style={{ transformOrigin: "50px 90px" }}
          animate={animated ? { scaleY: [1, 1 + floatHeight * 0.03, 1 - floatHeight * 0.02, 1] } : { scaleY: 1 }}
          transition={{ duration: floatDuration, repeat: Infinity, ease: "easeInOut" }}
        >
          <Spring mini={mini} />
        </motion.g>
        <Body armAngle={armAngle} showArms={!mini} />
        <Gem />
        <Face mood={mood} blinking={blinking} look={look} mini={mini} />
        <Accessories mood={mood} animated={animated} speed={speed} />
      </motion.g>
    </svg>
  );
}
