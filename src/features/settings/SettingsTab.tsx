import { Bell, Gauge, Ghost as GhostIcon, Lock, PanelTop, RotateCcw, Timer } from "lucide-react";
import { BentoCard } from "../../components/ui/BentoCard";
import { Toggle } from "../../components/ui/Toggle";
import { useSettings } from "../../store/SettingsContext";
import { AutostartToggle } from "./AutostartToggle";
import { BlockedApps } from "./BlockedApps";
import { HotkeySetting } from "./HotkeySetting";
import { NotificationAccess } from "./NotificationAccess";
import { SettingRow } from "./SettingRow";
import { SettingSlider } from "./SettingSlider";

const ms = (value: number) => `${value} ms`;
const minutes = (value: number) => `${value} min`;
const seconds = (value: number) => (value === 0 ? "Jamais" : `${value} s`);

export function SettingsTab() {
  const { settings, update, reset, error } = useSettings();
  const { animations, island, timer, notifications } = settings;

  return (
    <div className="flex flex-col gap-3">
      <div className="grid grid-cols-2 gap-3">
        <div className="flex flex-col gap-3">
          <BentoCard title="Animations" icon={<Gauge size={14} />}>
            <SettingRow label="Animations activées" description="Désactive tous les mouvements si coupé.">
              <Toggle
                label="Animations activées"
                checked={animations.enabled}
                onChange={(enabled) => update((s) => ({ ...s, animations: { ...s.animations, enabled } }))}
              />
            </SettingRow>
            <SettingSlider
              label="Vitesse"
              description="Plus élevé, plus rapide."
              value={Math.round(animations.speed * 100)}
              min={25}
              max={300}
              step={5}
              format={(value) => `×${(value / 100).toFixed(2)}`}
              onChange={(value) => update((s) => ({ ...s, animations: { ...s.animations, speed: value / 100 } }))}
            />
            <SettingSlider
              label="Délai d'ouverture"
              description="Survol avant d'ouvrir le menu."
              value={animations.openDelayMs}
              min={0}
              max={1500}
              step={50}
              format={ms}
              onChange={(openDelayMs) => update((s) => ({ ...s, animations: { ...s.animations, openDelayMs } }))}
            />
            <SettingSlider
              label="Délai de fermeture"
              description="Attente après avoir quitté le menu."
              value={animations.closeDelayMs}
              min={0}
              max={3000}
              step={50}
              format={ms}
              onChange={(closeDelayMs) => update((s) => ({ ...s, animations: { ...s.animations, closeDelayMs } }))}
            />
            <SettingSlider
              label="Changement d'onglet"
              description="Durée du fondu entre deux onglets."
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
              label="Disparition automatique"
              description="Délai avant que l'Island se cache. 0 = jamais."
              value={island.autoHideSeconds}
              min={0}
              max={120}
              step={5}
              format={seconds}
              onChange={(autoHideSeconds) => update((s) => ({ ...s, island: { ...s.island, autoHideSeconds } }))}
            />
            <SettingRow label="Média dans la pilule" description="Pochette et onde sonore du média en cours.">
              <Toggle
                label="Afficher le média dans la pilule"
                checked={island.showMediaInPill}
                onChange={(showMediaInPill) => update((s) => ({ ...s, island: { ...s.island, showMediaInPill } }))}
              />
            </SettingRow>
          </BentoCard>

          <BentoCard title="Mascotte" icon={<GhostIcon size={14} />}>
            <SettingRow label="Content après une tâche" description="Speedtest ou nettoyage de la RAM réussi.">
              <Toggle
                label="Content après une tâche"
                checked={settings.mascot.happyOnTasks}
                onChange={(happyOnTasks) => update((s) => ({ ...s, mascot: { ...s.mascot, happyOnTasks } }))}
              />
            </SettingRow>
            <SettingRow label="Concentré en plein écran" description="Une application occupe tout l'écran.">
              <Toggle
                label="Concentré en plein écran"
                checked={settings.mascot.focusOnFullscreen}
                onChange={(focusOnFullscreen) => update((s) => ({ ...s, mascot: { ...s.mascot, focusOnFullscreen } }))}
              />
            </SettingRow>
            <SettingRow label="Endormi la nuit">
              <Toggle
                label="Endormi la nuit"
                checked={settings.mascot.sleepAtNight}
                onChange={(sleepAtNight) => update((s) => ({ ...s, mascot: { ...s.mascot, sleepAtNight } }))}
              />
            </SettingRow>
            <SettingSlider
              label="La nuit commence à"
              value={settings.mascot.nightStartHour}
              min={0}
              max={23}
              format={(hour) => `${hour} h`}
              onChange={(nightStartHour) => update((s) => ({ ...s, mascot: { ...s.mascot, nightStartHour } }))}
            />
            <SettingSlider
              label="La nuit finit à"
              value={settings.mascot.nightEndHour}
              min={0}
              max={23}
              format={(hour) => `${hour} h`}
              onChange={(nightEndHour) => update((s) => ({ ...s, mascot: { ...s.mascot, nightEndHour } }))}
            />
          </BentoCard>

          <BentoCard title="Presse-papier" icon={<Lock size={14} />}>
            <p className="text-[11px] leading-snug text-white/45">
              Les copies faites depuis un gestionnaire de mots de passe ne sont jamais enregistrées.
            </p>
            <SettingRow label="Détecter les secrets" description="Clés d'API, jetons, mots de passe copiés à la main.">
              <Toggle
                label="Détecter les secrets"
                checked={settings.clipboard.detectSecrets}
                onChange={(detectSecrets) => update((s) => ({ ...s, clipboard: { ...s.clipboard, detectSecrets } }))}
              />
            </SettingRow>
            <SettingSlider
              label="Effacer les secrets après"
              description="Masqués dans la liste, puis supprimés. Épingler les conserve."
              value={settings.clipboard.secretTtlMinutes}
              min={1}
              max={60}
              format={minutes}
              onChange={(secretTtlMinutes) => update((s) => ({ ...s, clipboard: { ...s.clipboard, secretTtlMinutes } }))}
            />
          </BentoCard>
        </div>

        <div className="flex flex-col gap-3">
          <BentoCard title="Minuteur et Pomodoro" icon={<Timer size={14} />}>
            <SettingSlider
              label="Concentration"
              value={timer.focusMinutes}
              min={1}
              max={120}
              format={minutes}
              onChange={(focusMinutes) => update((s) => ({ ...s, timer: { ...s.timer, focusMinutes } }))}
            />
            <SettingSlider
              label="Pause courte"
              value={timer.shortBreakMinutes}
              min={1}
              max={30}
              format={minutes}
              onChange={(shortBreakMinutes) => update((s) => ({ ...s, timer: { ...s.timer, shortBreakMinutes } }))}
            />
            <SettingSlider
              label="Pause longue"
              value={timer.longBreakMinutes}
              min={1}
              max={60}
              format={minutes}
              onChange={(longBreakMinutes) => update((s) => ({ ...s, timer: { ...s.timer, longBreakMinutes } }))}
            />
            <SettingSlider
              label="Sessions avant la pause longue"
              value={timer.sessionsBeforeLongBreak}
              min={1}
              max={8}
              format={(value) => `${value}`}
              onChange={(sessionsBeforeLongBreak) =>
                update((s) => ({ ...s, timer: { ...s.timer, sessionsBeforeLongBreak } }))
              }
            />
            <SettingRow label="Enchaîner automatiquement" description="Démarre la phase suivante sans clic.">
              <Toggle
                label="Enchaîner automatiquement"
                checked={timer.autoStartNext}
                onChange={(autoStartNext) => update((s) => ({ ...s, timer: { ...s.timer, autoStartNext } }))}
              />
            </SettingRow>
            <SettingRow label="Son en fin de phase">
              <Toggle
                label="Son en fin de phase"
                checked={timer.soundEnabled}
                onChange={(soundEnabled) => update((s) => ({ ...s, timer: { ...s.timer, soundEnabled } }))}
              />
            </SettingRow>
          </BentoCard>

          <BentoCard title="Notifications Windows" icon={<Bell size={14} />}>
            <NotificationAccess />
            <SettingRow label="Afficher dans l'Island">
              <Toggle
                label="Afficher les notifications dans l'Island"
                checked={notifications.enabled}
                onChange={(enabled) => update((s) => ({ ...s, notifications: { ...s.notifications, enabled } }))}
              />
            </SettingRow>
            <SettingSlider
              label="Durée d'affichage"
              value={notifications.displaySeconds}
              min={2}
              max={30}
              format={(value) => `${value} s`}
              onChange={(displaySeconds) =>
                update((s) => ({ ...s, notifications: { ...s.notifications, displaySeconds } }))
              }
            />
            <SettingRow label="Afficher le contenu" description="Sinon seul le nom de l'application apparaît.">
              <Toggle
                label="Afficher le contenu des notifications"
                checked={notifications.showBody}
                onChange={(showBody) => update((s) => ({ ...s, notifications: { ...s.notifications, showBody } }))}
              />
            </SettingRow>
            <SettingRow label="Son">
              <Toggle
                label="Son à l'arrivée d'une notification"
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
          Valeurs par défaut
        </button>
      </div>
    </div>
  );
}
