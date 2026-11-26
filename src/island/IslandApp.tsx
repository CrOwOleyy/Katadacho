import { listen } from "@tauri-apps/api/event";
import { motion, AnimatePresence } from "motion/react";
import { Activity, ClipboardList, Search, Settings as SettingsIcon, SlidersHorizontal, Timer as TimerIcon, X } from "lucide-react";
import { useEffect, useRef, useState, type ComponentType } from "react";
import { ClipboardTab } from "../features/clipboard/ClipboardTab";
import { ControlsTab } from "../features/controls/ControlsTab";
import { DashboardTab } from "../features/dashboard/DashboardTab";
import { LauncherTab } from "../features/launcher/LauncherTab";
import { SettingsTab } from "../features/settings/SettingsTab";
import { AiQuestionCard } from "../features/ai/AiQuestionCard";
import { NotificationBanner } from "../features/notifications/NotificationBanner";
import { TimerChip } from "../features/timer/TimerChip";
import { TimerTab } from "../features/timer/TimerTab";
import { useNotifications } from "../hooks/useNotifications";
import { useAiQuestion } from "../hooks/useAiQuestion";
import { useFullscreenApp } from "../hooks/useFullscreenApp";
import { useVideoApp } from "../hooks/useVideoApp";
import { useMoodPulse } from "../hooks/useMoodPulse";
import { useSystemIdle } from "../hooks/useSystemIdle";
import { isNight } from "../lib/night";
import { useTimer } from "../hooks/useTimer";
import type { GhostMood } from "../components/mascot/mood";
import { pickMood } from "../components/mascot/pickMood";
import { Soul } from "../components/soul/Soul";
import { MediaPill } from "../features/media/MediaPill";
import { useMedia } from "../hooks/useMedia";
import { useClickThroughBounds } from "../hooks/useClickThrough";
import { useMotionConfig } from "../hooks/useMotion";
import { useSettings } from "../store/SettingsContext";

const HIDDEN = { width: 200, height: 4 };
const IDLE_HEIGHT = 36;
/** Taille de la pilule pendant l'affichage d'une notification. */
const BANNER = { width: 460, height: 68 };
/** Taille de la pilule quand Claude pose une question avec des boutons. */
const QUESTION = { width: 520, height: 124 };
/** Largeur de la pilule repliée : base, plus la place du média et du minuteur quand ils sont affichés. */
const IDLE_BASE_WIDTH = 180;
const IDLE_MEDIA_EXTRA = 90;
const IDLE_TIMER_EXTRA = 74;
const EXPANDED = { width: 760, height: 480 };
const CLOCK_TICK_MS = 30_000;
/** Inactivité au bout de laquelle la mascotte s'endort. */
const SLEEP_AFTER_SECONDS = 60;
/** Fermeture d'un menu ouvert au clavier si la souris n'est jamais venue dessus. */
const ABANDONED_CLOSE_MS = 20_000;

type Tab = "controls" | "dashboard" | "clipboard" | "launcher" | "timer" | "settings";

const TABS: { id: Tab; label: string; icon: ComponentType<{ size?: number }> }[] = [
  { id: "controls", label: "Controls", icon: SlidersHorizontal },
  { id: "dashboard", label: "Dashboard", icon: Activity },
  { id: "clipboard", label: "Clipboard", icon: ClipboardList },
  { id: "launcher", label: "Launcher", icon: Search },
  { id: "timer", label: "Timer", icon: TimerIcon },
  { id: "settings", label: "Settings", icon: SettingsIcon },
];

function useClock() {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const id = window.setInterval(() => setNow(new Date()), CLOCK_TICK_MS);
    return () => window.clearInterval(id);
  }, []);
  return now.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
}

/** Vrai si un champ du menu contient du texte en cours de saisie. */
function isTyping(container: HTMLElement | null): boolean {
  const active = document.activeElement;
  if (!container || !active || !container.contains(active)) return false;
  return (
    (active instanceof HTMLInputElement || active instanceof HTMLTextAreaElement) && active.value !== ""
  );
}

export function IslandApp() {
  const [visible, setVisible] = useState(true);
  const [expanded, setExpanded] = useState(false);
  const [tab, setTab] = useState<Tab>("controls");
  const [launcherFocus, setLauncherFocus] = useState(0);
  const pillRef = useRef<HTMLDivElement>(null);
  const pointerInside = useRef(false);
  const hoverTimer = useRef<number | undefined>(undefined);
  const time = useClock();
  const mediaInfo = useMedia();
  const { settings } = useSettings();
  const { spring, fade, tabSeconds } = useMotionConfig();
  const { openDelayMs, closeDelayMs } = settings.animations;
  const { autoHideSeconds, showMediaInPill } = settings.island;
  const media = showMediaInPill ? mediaInfo : null;
  const { snapshot: timer, finished } = useTimer();
  const { current: toast, pending, dismissCurrent } = useNotifications();
  const { question: aiQuestion, answer: answerAiQuestion, error: aiQuestionError } = useAiQuestion();

  // Sans clavier ni souris depuis une minute, la mascotte s'endort.
  const userIdle = useSystemIdle(SLEEP_AFTER_SECONDS);

  const mascotSettings = settings.mascot;
  const moodPulse = useMoodPulse();
  const fullscreenApp = useFullscreenApp(mascotSettings.focusOnFullscreen);
  const videoApp = useVideoApp(mascotSettings.watchVideos);
  const night =
    mascotSettings.sleepAtNight &&
    isNight(new Date().getHours(), mascotSettings.nightStartHour, mascotSettings.nightEndHour);

  const mood: GhostMood = pickMood({
    notification: toast !== null || aiQuestion !== null,
    pulse: mascotSettings.happyOnTasks || moodPulse === "sad" ? moodPulse : null,
    timerFinished: finished !== null,
    timerRunning: timer.status === "running",
    timerPhase: timer.phase,
    fullscreenApp,
    videoApp,
    mediaPlaying: media?.playing ?? false,
    userIdle,
    night,
  });
  const timerVisible = timer.status !== "idle";

  useClickThroughBounds(pillRef);

  const clearHoverTimer = () => window.clearTimeout(hoverTimer.current);

  // L'Island se cache après le délai réglé une fois repliée (0 = jamais). Tant que le menu est
  // déployé ou qu'un minuteur est actif, le compte à rebours est suspendu.
  useEffect(() => {
    if (!visible || expanded || autoHideSeconds === 0 || timerVisible || toast !== null || aiQuestion !== null) return;
    const timeout = window.setTimeout(() => setVisible(false), autoHideSeconds * 1000);
    return () => window.clearTimeout(timeout);
  }, [visible, expanded, autoHideSeconds, timerVisible, toast, aiQuestion]);

  // Une phase du minuteur se termine : l'Island réapparaît, même si elle était cachée.
  useEffect(() => {
    if (finished) setVisible(true);
  }, [finished]);

  // Une question de Claude arrive : l'Island réapparaît, même si elle était cachée.
  const aiQuestionId = aiQuestion?.id;
  useEffect(() => {
    if (aiQuestionId !== undefined) setVisible(true);
  }, [aiQuestionId]);

  // Une notification arrive : l'Island réapparaît, même si elle était cachée.
  const toastId = toast?.id;
  useEffect(() => {
    if (toastId !== undefined) setVisible(true);
  }, [toastId]);

  // Un menu ouvert au clavier (Alt+Space) puis abandonné se referme tout seul.
  useEffect(() => {
    if (!expanded) return;
    const timeout = window.setTimeout(() => {
      if (!pointerInside.current && !isTyping(pillRef.current)) setExpanded(false);
    }, ABANDONED_CLOSE_MS);
    return () => window.clearTimeout(timeout);
  }, [expanded, launcherFocus]);

  // Échap ferme le menu déployé sans cacher l'Island.
  useEffect(() => {
    if (!expanded) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setExpanded(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [expanded]);

  // Le raccourci global (Alt+Space) affiche l'Island sur le lanceur, prête à recevoir la frappe.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let active = true;
    listen("launcher-hotkey", () => {
      clearHoverTimer();
      setVisible(true);
      setExpanded(true);
      setTab("launcher");
      setLauncherFocus((token) => token + 1);
    })
      .then((fn) => {
        if (active) unlisten = fn;
        else fn();
      })
      .catch((reason: unknown) => console.error("launcher-hotkey listener failed", reason));
    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  // L'icône de la zone de notification ouvre l'Island, éventuellement sur un onglet précis.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let active = true;
    listen<string | null>("island-open", (event) => {
      clearHoverTimer();
      setVisible(true);
      setExpanded(true);
      const requested = TABS.find((item) => item.id === event.payload);
      if (requested) setTab(requested.id);
    })
      .then((fn) => {
        if (active) unlisten = fn;
        else fn();
      })
      .catch((reason: unknown) => console.error("island-open listener failed", reason));
    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  useEffect(() => clearHoverTimer, []);

  const handleEnter = () => {
    pointerInside.current = true;
    clearHoverTimer();
    // Depuis la barre cachée, l'Island réapparaît tout de suite, le menu suit si le survol continue.
    if (!visible) setVisible(true);
    if (!expanded) {
      // Pendant une notification, le survol sert à la lire : il n'ouvre pas le menu.
      if (toast || aiQuestion) return;
      hoverTimer.current = window.setTimeout(() => setExpanded(true), openDelayMs);
    }
  };

  const handleLeave = () => {
    pointerInside.current = false;
    clearHoverTimer();
    if (!expanded) return;
    hoverTimer.current = window.setTimeout(() => {
      if (!pointerInside.current && !isTyping(pillRef.current)) setExpanded(false);
    }, closeDelayMs);
  };

  const idleSize = {
    width: IDLE_BASE_WIDTH + (media ? IDLE_MEDIA_EXTRA : 0) + (timerVisible ? IDLE_TIMER_EXTRA : 0),
    height: IDLE_HEIGHT,
  };
  const size = !visible ? HIDDEN : expanded ? EXPANDED : aiQuestion ? QUESTION : toast ? BANNER : idleSize;

  return (
    <div className="flex h-full w-full justify-center">
      <motion.div
        ref={pillRef}
        onMouseEnter={handleEnter}
        onMouseLeave={handleLeave}
        onClick={() => {
          // Un clic est une intention explicite : on ouvre sans attendre le délai de survol.
          if (visible && !expanded && !toast && !aiQuestion) {
            clearHoverTimer();
            setExpanded(true);
          }
        }}
        animate={{ width: size.width, height: size.height }}
        transition={spring}
        className={`relative flex flex-col overflow-hidden border border-white/10 shadow-2xl ${
          visible ? "bg-[color:var(--color-island)]" : "bg-[color:var(--accent-color)]/50"
        } ${expanded ? "rounded-[28px]" : "rounded-full"}`}
      >
        <AnimatePresence mode="wait" initial={false}>
          {visible && expanded && (
            <motion.div
              key="expanded"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={fade}
              className="flex h-full flex-col"
            >
              <header className="flex shrink-0 items-center justify-between px-4 pb-1 pt-3">
                <div className="flex items-center gap-2">
                  <Soul size={42} mood={mood} />
                  <nav className="ml-1 flex items-center gap-0.5" aria-label="Sections">
                    {TABS.map((item) => {
                      const Icon = item.icon;
                      const active = tab === item.id;
                      return (
                        <button
                          key={item.id}
                          type="button"
                          onClick={() => setTab(item.id)}
                          className={`relative rounded-lg px-3 py-1.5 text-xs font-medium transition-colors ${
                            active ? "text-white" : "text-white/50 hover:text-white"
                          }`}
                        >
                          {active && (
                            <motion.span
                              layoutId="active-tab"
                              transition={spring}
                              className="absolute inset-0 rounded-lg bg-white/12"
                            />
                          )}
                          <span className="relative flex items-center gap-1.5">
                            <Icon size={13} />
                            {item.label}
                          </span>
                        </button>
                      );
                    })}
                  </nav>
                </div>
                <button
                  type="button"
                  aria-label="Close menu"
                  onClick={() => setExpanded(false)}
                  className="rounded-lg p-1.5 text-white/60 transition-colors hover:bg-white/10 hover:text-white"
                >
                  <X size={16} />
                </button>
              </header>

              <div className="min-h-0 flex-1 overflow-y-auto px-4 pb-4 pt-2 [scrollbar-width:thin]">
                <AnimatePresence mode="wait" initial={false}>
                  <motion.div
                    key={tab}
                    initial={{ opacity: 0, y: 8 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -4 }}
                    transition={{ duration: tabSeconds }}
                  >
                    {tab === "controls" && <ControlsTab mood={mood} />}
                    {tab === "dashboard" && <DashboardTab />}
                    {tab === "clipboard" && <ClipboardTab />}
                    {tab === "timer" && <TimerTab snapshot={timer} finished={finished} />}
                    {tab === "settings" && <SettingsTab />}
                    {tab === "launcher" && (
                      <LauncherTab focusToken={launcherFocus} onDone={() => setExpanded(false)} />
                    )}
                  </motion.div>
                </AnimatePresence>
              </div>
            </motion.div>
          )}
          {visible && !expanded && aiQuestion && (
            <motion.div
              key="ai-question"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={fade}
              className="h-full w-full"
            >
              <AiQuestionCard question={aiQuestion} error={aiQuestionError} onAnswer={(option) => void answerAiQuestion(option)} />
            </motion.div>
          )}
          {visible && !expanded && !aiQuestion && toast && (
            <motion.div
              key="toast"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={fade}
              className="h-full w-full"
            >
              <NotificationBanner
                toast={toast}
                pending={pending}
                showBody={settings.notifications.showBody}
                onDismiss={dismissCurrent}
              />
            </motion.div>
          )}
          {visible && !expanded && !aiQuestion && !toast && (
            <motion.div
              key="idle"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={fade}
              className="h-full w-full cursor-pointer"
            >
              <MediaPill
                media={media}
                time={time}
                mood={mood}
                timerChip={timerVisible ? <TimerChip snapshot={timer} flashing={finished !== null} /> : null}
              />
            </motion.div>
          )}
        </AnimatePresence>
      </motion.div>
    </div>
  );
}
