import { listen } from "@tauri-apps/api/event";
import { ChevronDown } from "lucide-react";
import { useEffect, useState } from "react";
import { ipc } from "../../lib/ipc";
import type { HotkeyStatus } from "../../lib/types/hotkeys";
import type { LauncherHotkey } from "../../lib/types/settings";
import { useSettings } from "../../store/SettingsContext";
import { SettingRow } from "./SettingRow";

const OPTIONS: { value: LauncherHotkey; label: string }[] = [
  { value: "copilot", label: "Touche Copilot" },
  { value: "alt+space", label: "Alt + Espace" },
  { value: "ctrl+alt+space", label: "Ctrl + Alt + Espace" },
  { value: "ctrl+shift+space", label: "Ctrl + Maj + Espace" },
];

/** Choix du raccourci du lanceur, avec l'état réellement appliqué par Windows. */
export function HotkeySetting() {
  const { settings, update } = useSettings();
  const [status, setStatus] = useState<HotkeyStatus | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let active = true;

    ipc
      .getHotkeyStatus()
      .then((current) => {
        if (active) setStatus(current);
      })
      .catch((reason: unknown) => console.error("get_hotkey_status failed", reason));

    listen<HotkeyStatus>("hotkey-status", (event) => setStatus(event.payload))
      .then((fn) => {
        if (active) unlisten = fn;
        else fn();
      })
      .catch((reason: unknown) => console.error("hotkey-status listener failed", reason));

    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  return (
    <div className="flex flex-col gap-1">
      <SettingRow
        label="Ouvrir le lanceur"
        description={
          settings.hotkeys.launcher === "copilot"
            ? "Remplace l'ouverture de Copilot par le lanceur."
            : "Raccourci global, depuis n'importe quelle application."
        }
      >
        <div className="relative">
          <select
            aria-label="Raccourci du lanceur"
            value={settings.hotkeys.launcher}
            onChange={(event) =>
              update((s) => ({ ...s, hotkeys: { ...s.hotkeys, launcher: event.target.value as LauncherHotkey } }))
            }
            className="appearance-none rounded-lg border border-white/10 bg-white/5 py-1.5 pl-3 pr-8 text-xs text-white outline-none focus:border-[color:var(--accent-color)]"
          >
            {OPTIONS.map((option) => (
              <option key={option.value} value={option.value} className="bg-[#121216]">
                {option.label}
              </option>
            ))}
          </select>
          <ChevronDown size={13} className="pointer-events-none absolute right-2.5 top-1/2 -translate-y-1/2 text-white/50" />
        </div>
      </SettingRow>
      {status && status.error && <p className="text-[10px] text-red-400">{status.error}</p>}
      {status && status.active && status.choice === settings.hotkeys.launcher && (
        <p className="text-[10px] text-emerald-300/80">Raccourci actif</p>
      )}
    </div>
  );
}
