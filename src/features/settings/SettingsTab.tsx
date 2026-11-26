import { Bell, Gauge, Ghost as GhostIcon, Lock, PanelTop, RotateCcw, Timer } from "lucide-react";
import { BentoCard } from "../../components/ui/BentoCard";
import { Toggle } from "../../components/ui/Toggle";
import { useSettings } from "../../store/SettingsContext";
import { AutostartToggle } from "./AutostartToggle";
import { BlockedApps } from "./BlockedApps";
import { ClaudeBridgeSetting } from "./ClaudeBridgeSetting";
import { HotkeySetting } from "./HotkeySetting";
import { NotificationAccess } from "./NotificationAccess";
import { SettingRow } from "./SettingRow";
import { SettingSlider } from "./SettingSlider";

const ms = (value: number) => `${value} ms`;
const minutes = (value: number) => `${value} min`;
const seconds = (value: number) => (value === 0 ? "Never" : `${value} s`);

export function SettingsTab() {
  const { settings, update, reset, error } = useSettings();
  const { animations, island, timer, notifications } = settings;

  return (
    <div className="flex flex-col gap-3">
      <div className="grid grid-cols-2 gap-3">
        <div className="flex flex-col gap-3">
          <BentoCard title="Animations" icon={<Gauge size={14} />}>
            <SettingRow label="Animations enabled" description="Turns all motion off when disabled.">
              <Toggle
                label="Animations enabled"
                checked={animations.enabled}
                onChange={(enabled) => update((s) => ({ ...s, animations: { ...s.animations, enabled } }))}
              />
            </SettingRow>
            <SettingSlider
              label="Speed"
              description="Higher is faster."
              value={Math.round(animations.speed * 100)}
              min={25}
              max={300}
              step={5}
              format={(value) => `×${(value / 100).toFixed(2)}`}
              onChange={(value) => update((s) => ({ ...s, animations: { ...s.animations, speed: value / 100 } }))}
            />
            <SettingSlider
              label="Open delay"
              description="Hover time before the menu opens."
              value={animations.openDelayMs}
              min={0}
              max={1500}
              step={50}
              format={ms}
              onChange={(openDelayMs) => update((s) => ({ ...s, animations: { ...s.animations, openDelayMs } }))}
            />
            <SettingSlider
              label="Close delay"
              description="Wait after the pointer leaves the menu."
              value={animations.closeDelayMs}
              min={0}
              max={3000}
              step={50}
              format={ms}
              onChange={(closeDelayMs) => update((s) => ({ ...s, animations: { ...s.animations, closeDelayMs } }))}
            />
            <SettingSlider
              label="Tab switch"
              description="Fade duration between tabs."
              value={animations.tabTransitionMs}
              min={0}
              max={600}
              step={20}
              format={ms}
              onChange={(tabTransitionMs) => update((s) => ({ ...s, animations: { ...s.animations, tabTransitionMs } }))}
            />
          </BentoCard>

          <BentoCard title="Island" icon={<PanelTop size={14} />}>
            <AutostartToggle />
            <HotkeySetting />
            <SettingSlider
              label="Auto-hide"
              description="Delay before the island hides. 0 = never."
              value={island.autoHideSeconds}
              min={0}
              max={120}
              step={5}
              format={seconds}
              onChange={(autoHideSeconds) => update((s) => ({ ...s, island: { ...s.island, autoHideSeconds } }))}
            />
            <SettingRow label="Media in the pill" description="Cover art and waveform of what is playing.">
              <Toggle
                label="Show media in the pill"
                checked={island.showMediaInPill}
                onChange={(showMediaInPill) => update((s) => ({ ...s, island: { ...s.island, showMediaInPill } }))}
              />
            </SettingRow>
          </BentoCard>

          <ClaudeBridgeSetting />

          <BentoCard title="Mascot" icon={<GhostIcon size={14} />}>
            <SettingRow label="Happy after a task" description="After a successful speed test or RAM clean.">
              <Toggle
                label="Happy after a task"
                checked={settings.mascot.happyOnTasks}
                onChange={(happyOnTasks) => update((s) => ({ ...s, mascot: { ...s.mascot, happyOnTasks } }))}
              />
            </SettingRow>
            <SettingRow label="Focused in fullscreen" description="When an app fills the whole screen.">
              <Toggle
                label="Focused in fullscreen"
                checked={settings.mascot.focusOnFullscreen}
                onChange={(focusOnFullscreen) => update((s) => ({ ...s, mascot: { ...s.mascot, focusOnFullscreen } }))}
              />
            </SettingRow>
            <SettingRow label="Watching" description="A video or show playing in a player or on a streaming site.">
              <Toggle
                label="Watching"
                checked={settings.mascot.watchVideos}
                onChange={(watchVideos) => update((s) => ({ ...s, mascot: { ...s.mascot, watchVideos } }))}
              />
            </SettingRow>
            <SettingRow label="Asleep at night">
              <Toggle
                label="Asleep at night"
                checked={settings.mascot.sleepAtNight}
                onChange={(sleepAtNight) => update((s) => ({ ...s, mascot: { ...s.mascot, sleepAtNight } }))}
              />
            </SettingRow>
            <SettingSlider
              label="Night starts at"
              value={settings.mascot.nightStartHour}
              min={0}
              max={23}
              format={(hour) => `${hour}:00`}
              onChange={(nightStartHour) => update((s) => ({ ...s, mascot: { ...s.mascot, nightStartHour } }))}
            />
            <SettingSlider
              label="Night ends at"
              value={settings.mascot.nightEndHour}
              min={0}
              max={23}
              format={(hour) => `${hour}:00`}
              onChange={(nightEndHour) => update((s) => ({ ...s, mascot: { ...s.mascot, nightEndHour } }))}
            />
          </BentoCard>

          <BentoCard title="Clipboard" icon={<Lock size={14} />}>
            <p className="text-[11px] leading-snug text-white/45">
              Copies made from a password manager are never saved.
            </p>
            <SettingRow label="Detect secrets" description="API keys, tokens and passwords copied by hand.">
              <Toggle
                label="Detect secrets"
                checked={settings.clipboard.detectSecrets}
                onChange={(detectSecrets) => update((s) => ({ ...s, clipboard: { ...s.clipboard, detectSecrets } }))}
              />
            </SettingRow>
            <SettingSlider
              label="Erase secrets after"
              description="Hidden in the list, then deleted. Pinning keeps them."
              value={settings.clipboard.secretTtlMinutes}
              min={1}
              max={60}
              format={minutes}
              onChange={(secretTtlMinutes) => update((s) => ({ ...s, clipboard: { ...s.clipboard, secretTtlMinutes } }))}
            />
          </BentoCard>
        </div>

        <div className="flex flex-col gap-3">
          <BentoCard title="Timer and Pomodoro" icon={<Timer size={14} />}>
            <SettingSlider
              label="Focus"
              value={timer.focusMinutes}
              min={1}
              max={120}
              format={minutes}
              onChange={(focusMinutes) => update((s) => ({ ...s, timer: { ...s.timer, focusMinutes } }))}
            />
            <SettingSlider
              label="Short break"
              value={timer.shortBreakMinutes}
              min={1}
              max={30}
              format={minutes}
              onChange={(shortBreakMinutes) => update((s) => ({ ...s, timer: { ...s.timer, shortBreakMinutes } }))}
            />
            <SettingSlider
              label="Long break"
              value={timer.longBreakMinutes}
              min={1}
              max={60}
              format={minutes}
              onChange={(longBreakMinutes) => update((s) => ({ ...s, timer: { ...s.timer, longBreakMinutes } }))}
            />
            <SettingSlider
              label="Sessions before a long break"
              value={timer.sessionsBeforeLongBreak}
              min={1}
              max={8}
              format={(value) => `${value}`}
              onChange={(sessionsBeforeLongBreak) =>
                update((s) => ({ ...s, timer: { ...s.timer, sessionsBeforeLongBreak } }))
              }
            />
            <SettingRow label="Start the next phase automatically" description="No click needed between phases.">
              <Toggle
                label="Start the next phase automatically"
                checked={timer.autoStartNext}
                onChange={(autoStartNext) => update((s) => ({ ...s, timer: { ...s.timer, autoStartNext } }))}
              />
            </SettingRow>
            <SettingRow label="Sound at the end of a phase">
              <Toggle
                label="Sound at the end of a phase"
                checked={timer.soundEnabled}
                onChange={(soundEnabled) => update((s) => ({ ...s, timer: { ...s.timer, soundEnabled } }))}
              />
            </SettingRow>
          </BentoCard>

          <BentoCard title="Notifications" icon={<Bell size={14} />}>
            <NotificationAccess />
            <SettingRow label="Show in the island">
              <Toggle
                label="Show notifications in the island"
                checked={notifications.enabled}
                onChange={(enabled) => update((s) => ({ ...s, notifications: { ...s.notifications, enabled } }))}
              />
            </SettingRow>
            <SettingSlider
              label="Display time"
              value={notifications.displaySeconds}
              min={2}
              max={30}
              format={(value) => `${value} s`}
              onChange={(displaySeconds) =>
                update((s) => ({ ...s, notifications: { ...s.notifications, displaySeconds } }))
              }
            />
            <SettingRow label="Show content" description="Otherwise only the app name is shown.">
              <Toggle
                label="Show notification content"
                checked={notifications.showBody}
                onChange={(showBody) => update((s) => ({ ...s, notifications: { ...s.notifications, showBody } }))}
              />
            </SettingRow>
            <SettingRow label="Sound">
              <Toggle
                label="Sound when a notification arrives"
                checked={notifications.soundEnabled}
                onChange={(soundEnabled) =>
                  update((s) => ({ ...s, notifications: { ...s.notifications, soundEnabled } }))
                }
              />
            </SettingRow>
            <BlockedApps
              apps={notifications.blockedApps}
              onChange={(blockedApps) =>
                update((s) => ({ ...s, notifications: { ...s.notifications, blockedApps } }))
              }
            />
          </BentoCard>
        </div>
      </div>

      <div className="flex items-center justify-between">
        {error ? <p className="truncate text-xs text-red-400">{error}</p> : <span />}
        <button
          type="button"
          onClick={() => void reset()}
          className="flex items-center gap-1.5 rounded-lg bg-white/8 px-3 py-1.5 text-xs text-white/70 transition-colors hover:bg-white/12 hover:text-white"
        >
          <RotateCcw size={12} />
          Reset to defaults
        </button>
      </div>
    </div>
  );
}
