import { useEffect, useState } from "react";
import { ipc } from "../../lib/ipc";
import type { NotificationAccess as Access } from "../../lib/types/notifications";
import { SettingRow } from "./SettingRow";

const LABEL: Record<Access, string> = {
  allowed: "Allowed",
  denied: "Denied",
  unspecified: "Not requested",
};

/** État de l'autorisation Windows de lire les notifications, avec un bouton pour la demander. */
export function NotificationAccess() {
  const [access, setAccess] = useState<Access | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    ipc
      .getNotificationAccess()
      .then((status) => {
        if (active) setAccess(status);
      })
      .catch((reason: unknown) => {
        if (active) setError(String(reason));
      });
    return () => {
      active = false;
    };
  }, []);

  const request = async () => {
    setBusy(true);
    try {
      setAccess(await ipc.requestNotificationAccess());
      setError(null);
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-col gap-1">
      <SettingRow
        label="Notification access"
        description={
          access === "denied"
            ? "Allow access in Windows Settings, Privacy & security, Notifications."
            : "Needed to read notifications from other apps."
        }
      >
        <span
          className={`rounded-full px-2.5 py-0.5 text-[11px] font-medium ${
            access === "allowed" ? "bg-emerald-500/20 text-emerald-300" : "bg-white/10 text-white/60"
          }`}
        >
          {access ? LABEL[access] : "…"}
        </span>
        {access === "unspecified" && (
          <button
            type="button"
            disabled={busy}
            onClick={() => void request()}
            className="rounded-lg bg-[color:var(--accent-color)]/25 px-3 py-1 text-xs font-medium hover:bg-[color:var(--accent-color)]/35 disabled:opacity-50"
          >
            Allow
          </button>
        )}
      </SettingRow>
      {error && <p className="truncate text-[10px] text-red-400">{error}</p>}
    </div>
  );
}
