import { motion, AnimatePresence } from "motion/react";
import { X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { ControlsTab } from "../features/controls/ControlsTab";
import { DashboardTab } from "../features/dashboard/DashboardTab";
import { Soul } from "../components/soul/Soul";
import { useClickThroughBounds } from "../hooks/useClickThrough";
import { springSpring } from "../lib/motion";

const HIDDEN = { width: 240, height: 6 };
const IDLE = { width: 180, height: 36 };
const EXPANDED = { width: 640, height: 360 };
const CLOCK_TICK_MS = 30_000;

type Tab = "controls" | "dashboard";

const TABS: { id: Tab; label: string }[] = [
  { id: "controls", label: "Controls" },
  { id: "dashboard", label: "Dashboard" },
];
const VISIBLE_MS = 15_000;

function useClock() {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const id = window.setInterval(() => setNow(new Date()), CLOCK_TICK_MS);
    return () => window.clearInterval(id);
  }, []);
  return now.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
}

export function IslandApp() {
  const [visible, setVisible] = useState(true);
  const [expanded, setExpanded] = useState(false);
  const [tab, setTab] = useState<Tab>("controls");
  const pillRef = useRef<HTMLDivElement>(null);
  const time = useClock();

  useClickThroughBounds(pillRef);

  // L'Island disparaît 15 s après être repliée. Tant que le menu est déployé (souris dessus),
  // le compte à rebours est suspendu, donc le dashboard reste visible pendant qu'on le regarde.
  useEffect(() => {
    if (!visible || expanded) return;
    const timeout = window.setTimeout(() => setVisible(false), VISIBLE_MS);
    return () => window.clearTimeout(timeout);
  }, [visible, expanded]);

  // Échap ferme le menu déployé sans cacher l'Island.
  useEffect(() => {
    if (!expanded) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setExpanded(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [expanded]);

  // Survoler la barre cachée en haut fait réapparaître l'Island, déployée.
  const reveal = () => {
    setVisible(true);
    setExpanded(true);
  };

  const size = !visible ? HIDDEN : expanded ? EXPANDED : IDLE;

  return (
    <div className="flex h-full w-full justify-center">
      <motion.div
        ref={pillRef}
        onMouseEnter={reveal}
        onMouseLeave={() => setExpanded(false)}
        animate={{ width: size.width, height: size.height }}
        transition={springSpring}
        className={`relative flex flex-col overflow-hidden border border-white/10 bg-[color:var(--color-island)] shadow-2xl ${
          expanded ? "rounded-[28px]" : "rounded-full"
        }`}
      >
        <AnimatePresence mode="wait" initial={false}>
          {visible && expanded && (
            <motion.div
              key="expanded"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.15 }}
              className="flex h-full flex-col gap-3 overflow-y-auto p-4 [scrollbar-width:none]"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Soul size={20} />
                  {TABS.map((item) => (
                    <button
                      key={item.id}
                      type="button"
                      onClick={() => setTab(item.id)}
                      className={`rounded-lg px-2.5 py-1 text-xs font-medium transition-colors ${
                        tab === item.id ? "bg-white/15 text-white" : "text-white/50 hover:text-white"
                      }`}
                    >
                      {item.label}
                    </button>
                  ))}
                </div>
                <button
                  type="button"
                  aria-label="Fermer le menu"
                  onClick={() => setExpanded(false)}
                  className="rounded-lg p-1 text-white/60 transition-colors hover:bg-white/10 hover:text-white"
                >
                  <X size={16} />
                </button>
              </div>
              {tab === "controls" ? <ControlsTab /> : <DashboardTab />}
            </motion.div>
          )}
          {visible && !expanded && (
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
