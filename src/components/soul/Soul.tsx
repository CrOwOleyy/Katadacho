import { motion } from "motion/react";
import { useEffect, useState } from "react";

const BLINK_MIN_MS = 2500;
const BLINK_MAX_MS = 6000;
const BLINK_DURATION_MS = 140;

/** Orbe "Soul" : deux yeux qui clignent à intervalle aléatoire. */
export function Soul({ size = 22 }: { size?: number }) {
  const [blinking, setBlinking] = useState(false);

  useEffect(() => {
    let timeout = 0;
    let closeTimeout = 0;

    const schedule = () => {
      const delay = BLINK_MIN_MS + Math.random() * (BLINK_MAX_MS - BLINK_MIN_MS);
      timeout = window.setTimeout(() => {
        setBlinking(true);
        closeTimeout = window.setTimeout(() => {
          setBlinking(false);
          schedule();
        }, BLINK_DURATION_MS);
      }, delay);
    };

    schedule();
    return () => {
      window.clearTimeout(timeout);
      window.clearTimeout(closeTimeout);
    };
  }, []);

  return (
    <div
      className="flex items-center justify-center rounded-full bg-[color:var(--accent-color)]/20 ring-1 ring-[color:var(--accent-color)]/40"
      style={{ width: size, height: size }}
    >
      <div className="flex gap-[3px]">
        {[0, 1].map((eye) => (
          <motion.span
            key={eye}
            animate={{ scaleY: blinking ? 0.1 : 1 }}
            transition={{ duration: 0.07 }}
            className="block h-[6px] w-[3px] rounded-full bg-[color:var(--accent-color)]"
          />
        ))}
      </div>
    </div>
  );
}
