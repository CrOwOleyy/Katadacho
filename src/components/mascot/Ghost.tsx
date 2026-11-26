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
/** Intervalle aléatoire entre deux petites actions au repos. */
const ACTION_MIN_MS = 3500;
const ACTION_MAX_MS = 8000;
/** Le flottement est réglé en pixels réels, pour rester visible même quand le fantôme est petit. */
const FLOAT_PX = { calm: 2, excited: 4, sleepy: 1 };
/** Plafond en unités du dessin, pour que les grandes tailles ne s'envolent pas. */
const MAX_FLOAT_UNITS = 9;

/** Humeurs où les yeux sont des disques qui peuvent cligner. */
const BLINKING_MOODS: GhostMood[] = ["idle", "thinking", "music", "focus", "sad", "watching"];

type IdleAction = "lookLeft" | "lookRight" | "lookUp" | "hop" | "wiggle";
const IDLE_ACTIONS: IdleAction[] = ["lookLeft", "lookRight", "lookUp", "hop", "wiggle"];
/** Durée pendant laquelle une action reste visible avant le retour au repos. */
const ACTION_DURATION_MS: Record<IdleAction, number> = {
  lookLeft: 1400,
  lookRight: 1400,
  lookUp: 1200,
  hop: 600,
  wiggle: 900,
};
const LOOK: Record<IdleAction, { x: number; y: number }> = {
  lookLeft: { x: -3.5, y: 0.5 },
  lookRight: { x: 3.5, y: 0.5 },
  lookUp: { x: 0.5, y: -3 },
  hop: { x: 0, y: 0 },
  wiggle: { x: 0, y: 0 },
};

function randomBetween(min: number, max: number) {
  return min + Math.random() * (max - min);
}

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
      }, randomBetween(BLINK_MIN_MS, BLINK_MAX_MS));
    };

    schedule();
    return () => {
      window.clearTimeout(openTimer);
      window.clearTimeout(closeTimer);
    };
  }, [enabled]);

  return blinking;
}

/**
 * Au repos, le fantôme fait de temps en temps quelque chose : regarder autour de lui, sauter,
 * se dandiner. `key` change à chaque action, pour rejouer l'animation même si elle se répète.
 */
function useIdleAction(enabled: boolean): { action: IdleAction | null; key: number } {
  const [state, setState] = useState<{ action: IdleAction | null; key: number }>({ action: null, key: 0 });

  useEffect(() => {
    if (!enabled) {
      setState((current) => ({ action: null, key: current.key }));
      return;
    }
    let startTimer = 0;
    let endTimer = 0;

    const schedule = () => {
      startTimer = window.setTimeout(() => {
        const action = IDLE_ACTIONS[Math.floor(Math.random() * IDLE_ACTIONS.length)];
        setState((current) => ({ action, key: current.key + 1 }));
        endTimer = window.setTimeout(() => {
          setState((current) => ({ action: null, key: current.key }));
          schedule();
        }, ACTION_DURATION_MS[action]);
      }, randomBetween(ACTION_MIN_MS, ACTION_MAX_MS));
    };

    schedule();
    return () => {
      window.clearTimeout(startTimer);
      window.clearTimeout(endTimer);
    };
  }, [enabled]);

  return state;
}

export interface GhostProps {
  /** Hauteur en pixels. La largeur suit les proportions du dessin. */
  size?: number;
  mood?: GhostMood;
  /** Désactive le flottement, le clignement, les actions au repos et les accessoires animés. */
  animated?: boolean;
  speed?: number;
  /** Regard imposé de l'extérieur (par exemple vers la souris). Prioritaire sur les actions au repos. */
  look?: { x: number; y: number };
  className?: string;
}

export function Ghost({ size = 64, mood = "idle", animated = true, speed = 1, look, className }: GhostProps) {
  const mini = size < MINI_THRESHOLD;
  const blinking = useBlink(animated && BLINKING_MOODS.includes(mood));
  const { action, key: actionKey } = useIdleAction(animated && mood === "idle");

  const armAngle = mood === "happy" ? 125 : mood === "alert" ? 35 : action === "wiggle" ? 40 : 0;
  const excited = mood === "happy" || mood === "alert";
  const sleepy = mood === "sleepy";

  const floatPx = excited ? FLOAT_PX.excited : sleepy ? FLOAT_PX.sleepy : FLOAT_PX.calm;
  const floatUnits = Math.min(MAX_FLOAT_UNITS, (floatPx * VIEW_HEIGHT) / size);
  const floatDuration = (excited ? 0.8 : sleepy ? 4.2 : 2.6) / speed;
  // Balancement léger autour du bas du ressort, plus marqué quand il dort.
  const sway = sleepy ? 4 : excited ? 0 : 2.5;

  const gaze = look ?? (action ? LOOK[action] : { x: 0, y: 0 });

  return (
    <svg
      viewBox={`0 0 ${VIEW_WIDTH} ${VIEW_HEIGHT}`}
      width={(size * VIEW_WIDTH) / VIEW_HEIGHT}
      height={size}
      className={className}
      overflow="visible"
      role="img"
      aria-label="Ghost mascot"
    >
      <motion.g
        style={{ originX: 0.5, originY: 1 }}
        animate={animated ? { y: [0, -floatUnits, 0], rotate: [-sway, sway, -sway] } : { y: 0, rotate: 0 }}
        transition={{
          y: { duration: floatDuration, repeat: Infinity, ease: "easeInOut" },
          rotate: { duration: floatDuration * 2, repeat: Infinity, ease: "easeInOut" },
        }}
      >
        {/* Petit saut ou dandinement occasionnel, rejoué à chaque nouvelle action. */}
        <motion.g
          key={actionKey}
          style={{ originX: 0.5, originY: 1 }}
          animate={
            action === "hop"
              ? { y: [0, -14, 0, -4, 0] }
              : action === "wiggle"
                ? { rotate: [0, -8, 8, -5, 0] }
                : { y: 0, rotate: 0 }
          }
          transition={{ duration: action === "hop" ? 0.55 : 0.8, ease: "easeOut" }}
        >
          <motion.g
            style={{ originX: 0.5, originY: 0 }}
            animate={
              animated
                ? { scaleY: [1, 1 + floatUnits * 0.025, 1 - floatUnits * 0.015, 1] }
                : { scaleY: 1 }
            }
            transition={{ duration: floatDuration, repeat: Infinity, ease: "easeInOut" }}
          >
            <Spring mini={mini} />
          </motion.g>
          <Body armAngle={armAngle} showArms={!mini} />
          <Gem />
          <Face mood={mood} blinking={blinking} look={gaze} mini={mini} />
          <Accessories mood={mood} animated={animated} speed={speed} />
        </motion.g>
      </motion.g>
    </svg>
  );
}
