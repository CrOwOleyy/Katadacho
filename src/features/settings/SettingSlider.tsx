import { Slider } from "../../components/ui/Slider";
import { SettingRow } from "./SettingRow";

interface SettingSliderProps {
  label: string;
  description?: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  /** Texte affiché à droite du curseur, par exemple « 300 ms ». */
  format: (value: number) => string;
  onChange: (value: number) => void;
}

export function SettingSlider({ label, description, value, min, max, step = 1, format, onChange }: SettingSliderProps) {
  return (
    <SettingRow label={label} description={description}>
      <div className="w-40">
        <Slider value={value} min={min} max={max} step={step} label={label} onCommit={onChange} />
      </div>
      <span className="w-16 text-right font-mono text-xs text-white/60">{format(value)}</span>
    </SettingRow>
  );
}
