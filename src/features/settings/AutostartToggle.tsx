import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { useEffect, useState } from "react";
import { Toggle } from "../../components/ui/Toggle";
import { SettingRow } from "./SettingRow";

/**
 * Lancement automatique avec la session. L'état est lu dans le système lui-même, pas dans nos réglages :
 * il reste juste même si l'utilisateur le change depuis le Gestionnaire des tâches.
 */
export function AutostartToggle() {
  const [enabled, setEnabled] = useState<boolean | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    isEnabled()
      .then((value) => {
        if (active) setEnabled(value);
      })
      .catch((reason: unknown) => {
        if (active) setError(String(reason));
      });
    return () => {
      active = false;
    };
  }, []);

  const toggle = async (next: boolean) => {
    setBusy(true);
    try {
      if (next) await enable();
      else await disable();
      setEnabled(await isEnabled());
      setError(null);
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-col gap-1">
      <SettingRow label="Lancer au démarrage de la session" description="Katadacho démarre avec ta session.">
        <Toggle label="Lancer au démarrage de la session" checked={enabled} disabled={busy} onChange={(next) => void toggle(next)} />
      </SettingRow>
      {error && <p className="truncate text-[10px] text-red-400">{error}</p>}
    </div>
  );
}
