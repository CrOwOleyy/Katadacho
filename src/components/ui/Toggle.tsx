import { motion } from "motion/react";
import { useMotionConfig } from "../../hooks/useMotion";

interface ToggleProps {
  checked: boolean | null;
  disabled?: boolean;
  label: string;
  onChange: (next: boolean) => void;
}

export function Toggle({ checked, disabled = false, label, onChange }: ToggleProps) {
  const { spring } = useMotionConfig();
  const unavailable = checked === null;
  const isOn = checked === true;

  return (
    <button
      type="button"
      role="switch"
      aria-label={label}
      aria-checked={isOn}
      disabled={disabled || unavailable}
      onClick={() => onChange(!isOn)}
      className={`relative flex h-6 w-11 shrink-0 items-center rounded-full border border-white/10 p-0.5 transition-colors disabled:opacity-40 ${
        isOn ? "bg-[color:var(--accent-color)]" : "bg-white/10"
      }`}
    >
      <motion.span
        animate={{ x: isOn ? 20 : 0 }}
        transition={spring}
        className="h-5 w-5 rounded-full bg-white shadow"
      />
    </button>
  );
}
