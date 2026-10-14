import type { Transition } from "motion/react";

/** Ressort de référence du design : fluide, sans rebond excessif. */
export const springSpring: Transition = {
  type: "spring",
  stiffness: 400,
  damping: 30,
};
