import { motion, AnimatePresence } from "motion/react";
import { useEffect, useRef, useState } from "react";
import { ControlsTab } from "../features/controls/ControlsTab";
import { Soul } from "../components/soul/Soul";
import { useClickThroughBounds } from "../hooks/useClickThrough";
import { springSpring } from "../lib/motion";

const IDLE = { width: 180, height: 36 };
const EXPANDED = { width: 640, height: 360 };
const CLOCK_TICK_MS = 30_000;

function useClock() {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const id = window.setInterval(() => setNow(new Date()), CLOCK_TICK_MS);
    return () => window.clearInterval(id);
  }, []);
  return now.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
}

export function IslandApp() {
  const [expanded, setExpanded] = useState(false);
  const pillRef = useRef<HTMLDivElement>(null);
  const time = useClock();

  useClickThroughBounds(pillRef);

  const size = expanded ? EXPANDED : IDLE;

  return (
    <div className="flex h-full w-full justify-center">
      <motion.div
        ref={pillRef}
        onMouseEnter={() => setExpanded(true)}
        onMouseLeave={() => setExpanded(false)}
        animate={{ width: size.width, height: size.height }}
        transition={springSpring}
        className={`relative flex flex-col overflow-hidden border border-white/10 bg-[color:var(--color-island)] shadow-2xl ${
          expanded ? "rounded-[28px]" : "rounded-full"
        }`}
      >
        <AnimatePresence mode="wait" initial={false}>
          {expanded ? (
            <motion.div
              key="expanded"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.15 }}
              className="flex h-full flex-col gap-3 overflow-y-auto p-4 [scrollbar-width:none]"
            >
              <div className="flex items-center gap-2">
                <Soul size={20} />
                <span className="text-sm font-medium">Controls</span>
              </div>
              <ControlsTab />
            </motion.div>
          ) : (
            <motion.div
              key="idle"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.15 }}
              className="flex h-full items-center justify-between px-3"
            >
              <Soul />
              <span className="font-mono text-xs text-white/70">{time}</span>
            </motion.div>
          )}
        </AnimatePresence>
      </motion.div>
    </div>
  );
}
