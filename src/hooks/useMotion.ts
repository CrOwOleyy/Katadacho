import type { Transition } from "motion/react";
import { useMemo } from "react";
import { useSettings } from "../store/SettingsContext";

const BASE_STIFFNESS = 400;
const BASE_DAMPING = 30;
const INSTANT: Transition = { duration: 0 };

export interface MotionConfig {
  /** Ressort à utiliser pour les changements de taille et de position. */
  spring: Transition;
  /** Transition courte pour les fondus (changement d'onglet, apparition de contenu). */
  fade: Transition;
  /** Durée du fondu entre onglets, en secondes. */
  tabSeconds: number;
  enabled: boolean;
}

/**
 * Traduit les réglages d'animation en transitions Motion. Multiplier la vitesse par `s` multiplie
 * la raideur par s² et l'amortissement par s : même ressenti, en plus rapide ou plus lent.
 */
export function useMotionConfig(): MotionConfig {
  const { settings } = useSettings();
  const { enabled, speed, tabTransitionMs } = settings.animations;

  return useMemo(() => {
    if (!enabled) {
      return { spring: INSTANT, fade: INSTANT, tabSeconds: 0, enabled: false };
    }
    return {
      spring: {
        type: "spring",
        stiffness: BASE_STIFFNESS * speed * speed,
        damping: BASE_DAMPING * speed,
      },
      fade: { duration: 0.15 / speed },
      tabSeconds: tabTransitionMs / 1000,
      enabled: true,
    };
  }, [enabled, speed, tabTransitionMs]);
}
